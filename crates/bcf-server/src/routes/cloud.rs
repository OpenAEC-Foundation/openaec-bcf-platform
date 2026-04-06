//! Nextcloud cloud storage API routes via `openaec-cloud`.
//!
//! GET    /api/cloud/status                              -- connection status
//! GET    /api/cloud/projects                            -- list project folders
//! GET    /api/cloud/projects/{project}/files             -- list issue files
//! GET    /api/cloud/projects/{project}/files/{filename}  -- download file
//! PUT    /api/cloud/projects/{project}/files/{filename}  -- upload file
//! DELETE /api/cloud/projects/{project}/files/{filename}  -- delete file
//! GET    /api/cloud/projects/{project}/models            -- list IFC models
//! PUT    /api/cloud/projects/{project}/save/{project_id} -- export BCF to cloud
//! GET    /api/cloud/projects/{project}/manifest          -- read project manifest (default or named)
//! PUT    /api/cloud/projects/{project}/manifest          -- merge-update project manifest (default or named)
//! GET    /api/cloud/projects/{project}/manifests         -- list all .wefc manifests
//! GET    /api/cloud/projects/{project}/manifests/{name}  -- read a specific manifest by name

use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::sync::Arc;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::webdav::CloudClient;

/// Cloud storage routes.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/cloud/status", get(cloud_status))
        .route("/cloud/projects", get(cloud_list_projects))
        .route("/cloud/projects/{project}/files", get(cloud_list_files))
        .route(
            "/cloud/projects/{project}/files/{filename}",
            get(cloud_download_file)
                .put(cloud_upload_file)
                .delete(cloud_delete_file),
        )
        .route("/cloud/projects/{project}/models", get(cloud_list_models))
        .route(
            "/cloud/projects/{project}/save/{project_id}",
            put(cloud_save_bcf),
        )
        .route(
            "/cloud/projects/{project}/manifest",
            get(cloud_read_manifest).put(cloud_write_manifest),
        )
        .route(
            "/cloud/projects/{project}/manifests",
            get(cloud_list_manifests),
        )
        .route(
            "/cloud/projects/{project}/manifests/{name}",
            get(cloud_read_manifest_by_name),
        )
}

/// Response for cloud status check.
#[derive(Serialize)]
struct CloudStatus {
    enabled: bool,
    connected: bool,
}

/// GET /api/cloud/status
async fn cloud_status(State(state): State<AppState>) -> AppResult<Json<CloudStatus>> {
    let Some(ref client) = state.cloud else {
        return Ok(Json(CloudStatus {
            enabled: false,
            connected: false,
        }));
    };

    let connected = client.is_available().await;
    Ok(Json(CloudStatus {
        enabled: true,
        connected,
    }))
}

/// A project entry for the API response.
#[derive(Serialize)]
struct ProjectEntry {
    name: String,
}

/// Response wrapper for project list.
#[derive(Serialize)]
struct ProjectListResponse {
    projects: Vec<ProjectEntry>,
}

/// GET /api/cloud/projects
async fn cloud_list_projects(
    State(state): State<AppState>,
) -> AppResult<Json<ProjectListResponse>> {
    let client = require_cloud(&state)?;

    // Try volume mount first (fast, synchronous)
    let vol_projects = client.list_projects();
    if !vol_projects.is_empty() {
        let projects = vol_projects
            .into_iter()
            .map(|p| ProjectEntry { name: p.name })
            .collect();
        return Ok(Json(ProjectListResponse { projects }));
    }

    // Fallback to WebDAV (async)
    let webdav_projects = client.list_projects_webdav().await?;
    let projects = webdav_projects
        .into_iter()
        .map(|p| ProjectEntry { name: p.name })
        .collect();
    Ok(Json(ProjectListResponse { projects }))
}

/// A file entry for the API response.
#[derive(Serialize)]
struct FileEntry {
    name: String,
    size: u64,
    last_modified: String,
}

/// Response wrapper for file list.
#[derive(Serialize)]
struct FileListResponse {
    files: Vec<FileEntry>,
}

/// GET /api/cloud/projects/{project}/files
///
/// Lists files in the project's `issues/` directory (new structure)
/// with automatic fallback to `99_overige_documenten/bcf-platform/` (legacy).
async fn cloud_list_files(
    State(state): State<AppState>,
    Path(project): Path<String>,
) -> AppResult<Json<FileListResponse>> {
    let client = require_cloud(&state)?;

    // Try volume mount first
    let vol_files = client.list_files(&project);
    if !vol_files.is_empty() {
        let files = vol_files
            .into_iter()
            .map(|f| FileEntry {
                name: f.name,
                size: f.size,
                last_modified: f.last_modified,
            })
            .collect();
        return Ok(Json(FileListResponse { files }));
    }

    // Fallback to WebDAV listing
    let webdav_files = client.webdav.list_files(&project).await?;
    let files = webdav_files
        .into_iter()
        .map(|f| FileEntry {
            name: f.name,
            size: f.size,
            last_modified: f.last_modified,
        })
        .collect();
    Ok(Json(FileListResponse { files }))
}

/// GET /api/cloud/projects/{project}/files/{filename}
async fn cloud_download_file(
    State(state): State<AppState>,
    Path((project, filename)): Path<(String, String)>,
) -> AppResult<impl IntoResponse> {
    let client = require_cloud(&state)?;

    // Try volume mount read first, then WebDAV fallback
    let data = if let Some(bytes) = client.read_file(&project, "issues", &filename) {
        bytes
    } else {
        client.download_file(&project, &filename).await?
    };

    let content_type = if filename.ends_with(".bcfzip") {
        "application/zip"
    } else {
        "application/octet-stream"
    };

    Ok((
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        Body::from(data),
    ))
}

/// PUT /api/cloud/projects/{project}/files/{filename}
async fn cloud_upload_file(
    State(state): State<AppState>,
    Path((project, filename)): Path<(String, String)>,
    mut multipart: Multipart,
) -> AppResult<(StatusCode, Json<UploadResponse>)> {
    let client = require_cloud(&state)?;

    // Read file from multipart body
    let mut file_data: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
    {
        if field.name() == Some("file") {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("failed to read file: {e}")))?;
            file_data = Some(data.to_vec());
            break;
        }
    }

    let data =
        file_data.ok_or_else(|| AppError::BadRequest("missing 'file' field in multipart".to_string()))?;

    client.upload_file(&project, &filename, data).await?;

    Ok((
        StatusCode::CREATED,
        Json(UploadResponse {
            success: true,
            project,
            filename,
        }),
    ))
}

/// Response for successful upload.
#[derive(Serialize)]
struct UploadResponse {
    success: bool,
    project: String,
    filename: String,
}

/// DELETE /api/cloud/projects/{project}/files/{filename}
async fn cloud_delete_file(
    State(state): State<AppState>,
    Path((project, filename)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    let client = require_cloud(&state)?;
    client.delete_file(&project, &filename).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/cloud/projects/{project}/models
///
/// List IFC/BIM model files in the project's `models/` directory.
async fn cloud_list_models(
    State(state): State<AppState>,
    Path(project): Path<String>,
) -> AppResult<Json<FileListResponse>> {
    let client = require_cloud(&state)?;

    // Try volume mount
    let vol_models = client.list_models(&project);
    if !vol_models.is_empty() {
        let files = vol_models
            .into_iter()
            .map(|f| FileEntry {
                name: f.name,
                size: f.size,
                last_modified: f.last_modified,
            })
            .collect();
        return Ok(Json(FileListResponse { files }));
    }

    // Fallback to WebDAV listing at models/
    let webdav_models = client.webdav.list_path(&project, "models").await?;
    let files = webdav_models
        .into_iter()
        .filter(|f| {
            let lower = f.name.to_lowercase();
            lower.ends_with(".ifc")
                || lower.ends_with(".ifczip")
                || lower.ends_with(".ifcxml")
        })
        .map(|f| FileEntry {
            name: f.name,
            size: f.size,
            last_modified: f.last_modified,
        })
        .collect();
    Ok(Json(FileListResponse { files }))
}

/// Response for save operation.
#[derive(Serialize)]
struct SaveResponse {
    success: bool,
    project: String,
    filename: String,
}

/// PUT /api/cloud/projects/{project}/save/{project_id}
///
/// Export a BCF project from the database and save it to Nextcloud.
/// Uploads to `issues/` (new structure) and updates the project manifest.
async fn cloud_save_bcf(
    State(state): State<AppState>,
    Path((project_name, project_id)): Path<(String, Uuid)>,
) -> AppResult<Json<SaveResponse>> {
    let client = require_cloud(&state)?;

    // Verify project exists
    let project = db::projects::get_project(&state.pool, project_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("project {project_id}")))?;

    // Generate the BCF ZIP
    let zip_bytes = generate_bcfzip(&state, project_id).await?;

    let filename = format!(
        "{}.bcfzip",
        project.name.replace(' ', "_").to_lowercase()
    );

    // Upload to issues/ via WebDAV
    client
        .upload_file(&project_name, &filename, zip_bytes)
        .await?;

    // Read existing default manifest to find WefcModel references and existing IssueSet guid
    let issue_path = format!("issues/{filename}");
    let (model_refs, existing_guid, existing_created) =
        match client.read_default_manifest(&project_name).await {
            Ok(Some(manifest)) => {
                // Convert to serde_json::Value for inspection
                let manifest_json = serde_json::to_value(&manifest).unwrap_or_default();
                let refs = extract_model_refs(&manifest_json);
                let (guid, created) = find_existing_issue_set(&manifest_json, &issue_path);
                (refs, guid, created)
            }
            _ => (vec![], None, None),
        };

    // Reuse existing guid if updating, generate new one if creating
    let guid = existing_guid.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = chrono::Utc::now().to_rfc3339();
    let created = existing_created.unwrap_or_else(|| now.clone());

    // Update project.wefc manifest with a WefcIssueSet object
    let issue_set_object = serde_json::json!({
        "type": "WefcIssueSet",
        "guid": guid,
        "name": project.name,
        "version": "1.0.0",
        "status": "active",
        "created": created,
        "modified": now,
        "path": issue_path,
        "models": model_refs
    });

    // Best-effort manifest update (don't fail the save if manifest update fails)
    if let Err(e) = client
        .upsert_default_manifest_object(&project_name, issue_set_object)
        .await
    {
        tracing::warn!(
            project = %project_name,
            error = %e,
            "failed to update project manifest — BCF file was saved successfully"
        );
    }

    Ok(Json(SaveResponse {
        success: true,
        project: project_name,
        filename,
    }))
}

/// Optional query parameters for manifest endpoints.
#[derive(Deserialize)]
struct ManifestQuery {
    /// Manifest file name (default: `"project.wefc"`).
    manifest_name: Option<String>,
}

/// GET /api/cloud/projects/{project}/manifest
///
/// Read a project manifest as JSON. Accepts an optional `manifest_name`
/// query parameter to read a specific manifest (defaults to `"project.wefc"`).
async fn cloud_read_manifest(
    State(state): State<AppState>,
    Path(project): Path<String>,
    Query(query): Query<ManifestQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let client = require_cloud(&state)?;

    let manifest_opt = if let Some(ref name) = query.manifest_name {
        client.read_manifest(&project, name).await?
    } else {
        client.read_default_manifest(&project).await?
    };

    match manifest_opt {
        Some(manifest) => {
            let json = serde_json::to_value(&manifest)
                .map_err(|e| AppError::Internal(format!("manifest serialize: {e}")))?;
            Ok(Json(json))
        }
        None => Ok(Json(serde_json::json!({
            "header": null,
            "data": []
        }))),
    }
}

/// A manifest list entry for the API response.
#[derive(Serialize)]
struct ManifestEntry {
    name: String,
    size: u64,
    last_modified: String,
}

/// Response wrapper for manifest list.
#[derive(Serialize)]
struct ManifestListResponse {
    manifests: Vec<ManifestEntry>,
}

/// GET /api/cloud/projects/{project}/manifests
///
/// List all `.wefc` manifest files in a project directory.
async fn cloud_list_manifests(
    State(state): State<AppState>,
    Path(project): Path<String>,
) -> AppResult<Json<ManifestListResponse>> {
    let client = require_cloud(&state)?;

    let infos = client.list_manifests(&project).await?;
    let manifests = infos
        .into_iter()
        .map(|m| ManifestEntry {
            name: m.name,
            size: m.size,
            last_modified: m.last_modified,
        })
        .collect();
    Ok(Json(ManifestListResponse { manifests }))
}

/// GET /api/cloud/projects/{project}/manifests/{name}
///
/// Read a specific manifest by name.
async fn cloud_read_manifest_by_name(
    State(state): State<AppState>,
    Path((project, name)): Path<(String, String)>,
) -> AppResult<Json<serde_json::Value>> {
    let client = require_cloud(&state)?;

    match client.read_manifest(&project, &name).await? {
        Some(manifest) => {
            let json = serde_json::to_value(&manifest)
                .map_err(|e| AppError::Internal(format!("manifest serialize: {e}")))?;
            Ok(Json(json))
        }
        None => Err(AppError::NotFound(format!(
            "manifest '{name}' not found in project '{project}'"
        ))),
    }
}

/// Generate a BCF ZIP for a project (reused from bcfio).
async fn generate_bcfzip(state: &AppState, project_id: Uuid) -> AppResult<Vec<u8>> {
    let topic_rows = db::topics::list_topics(&state.pool, project_id).await?;
    let mut topic_folders = Vec::new();

    for topic_row in &topic_rows {
        let comment_rows = db::comments::list_comments(&state.pool, topic_row.id).await?;
        let comments: Vec<bcf_core::types::BcfComment> = comment_rows
            .into_iter()
            .map(|c| bcf_core::types::BcfComment {
                guid: c.id,
                comment: c.comment,
                author: None,
                viewpoint_guid: c.viewpoint_id,
                date: Some(c.created_at),
                modified_date: Some(c.updated_at),
                modified_author: None,
            })
            .collect();

        let vp_rows = db::viewpoints::list_viewpoints(&state.pool, topic_row.id).await?;
        let mut viewpoints = Vec::new();

        for vp_row in &vp_rows {
            let camera: Option<bcf_core::types::Camera> = vp_row
                .camera
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            let components: Option<bcf_core::types::Components> = vp_row
                .components
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok());

            let snapshot_data = if let Some(ref path) = vp_row.snapshot_path {
                state.storage.load(path).await.ok()
            } else {
                None
            };

            viewpoints.push(bcf_core::types::BcfViewpoint {
                guid: vp_row.id,
                camera,
                components,
                snapshot_data,
            });
        }

        let labels: Vec<String> =
            serde_json::from_value(topic_row.labels.clone()).unwrap_or_default();

        let bcf_topic = bcf_core::types::BcfTopic {
            guid: topic_row.id,
            title: topic_row.title.clone(),
            description: topic_row.description.clone(),
            topic_type: topic_row.topic_type.clone(),
            topic_status: topic_row.topic_status.clone(),
            priority: topic_row.priority.clone(),
            stage: topic_row.stage.clone(),
            labels,
            due_date: topic_row.due_date,
            assigned_to: None,
            creation_author: None,
            modified_author: None,
            creation_date: Some(topic_row.created_at),
            modified_date: Some(topic_row.updated_at),
            index: topic_row.index_number,
        };

        topic_folders.push(bcf_core::bcfzip::BcfTopicFolder {
            topic: bcf_topic,
            comments,
            viewpoints,
        });
    }

    let archive = bcf_core::bcfzip::BcfArchive {
        version: "2.1".to_string(),
        topics: topic_folders,
    };

    bcf_core::bcfzip::write_bcfzip(&archive)
        .map_err(|e| AppError::Internal(format!("BCF ZIP generation failed: {e}")))
}

/// Request body for PUT /manifest — a single WEFC data object to upsert.
#[derive(Deserialize)]
struct ManifestUpsertRequest {
    object: serde_json::Value,
}

/// PUT /api/cloud/projects/{project}/manifest
///
/// Merge-update: upsert a single data object into a project manifest.
/// Accepts an optional `manifest_name` query parameter (defaults to `"project.wefc"`).
/// Preserves all objects not owned by this tool. Updates existing objects
/// matched by `guid`, adds new ones. Updates the manifest header timestamp.
///
/// BCF Platform does NOT create manifests — it only updates existing ones.
/// Returns 404 if no manifest exists yet.
async fn cloud_write_manifest(
    State(state): State<AppState>,
    Path(project): Path<String>,
    Query(query): Query<ManifestQuery>,
    Json(body): Json<ManifestUpsertRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let client = require_cloud(&state)?;

    let manifest_name = query.manifest_name.as_deref().unwrap_or("project.wefc");

    // Verify manifest exists — BCF Platform never creates one
    let existing = client.read_manifest(&project, manifest_name).await?;
    if existing.is_none() {
        return Err(AppError::NotFound(format!(
            "manifest '{manifest_name}' does not exist — cannot update"
        )));
    }

    client
        .upsert_manifest_object(&project, manifest_name, body.object)
        .await?;

    // Return the updated manifest
    match client.read_manifest(&project, manifest_name).await? {
        Some(manifest) => {
            let json = serde_json::to_value(&manifest)
                .map_err(|e| AppError::Internal(format!("manifest serialize: {e}")))?;
            Ok(Json(json))
        }
        None => Ok(Json(serde_json::json!({
            "header": null,
            "data": []
        }))),
    }
}

/// Find an existing WefcIssueSet in the manifest by its `path` field.
///
/// Returns (guid, created) if found, so we can update rather than duplicate.
fn find_existing_issue_set(
    manifest: &serde_json::Value,
    path: &str,
) -> (Option<String>, Option<String>) {
    let empty = vec![];
    let data = manifest
        .get("data")
        .and_then(|d| d.as_array())
        .unwrap_or(&empty);

    for obj in data {
        let is_issue_set = obj
            .get("type")
            .and_then(|t| t.as_str())
            .map(|t| t == "WefcIssueSet")
            .unwrap_or(false);
        let matches_path = obj
            .get("path")
            .and_then(|p| p.as_str())
            .map(|p| p == path)
            .unwrap_or(false);

        if is_issue_set && matches_path {
            let guid = obj.get("guid").and_then(|g| g.as_str()).map(String::from);
            let created = obj
                .get("created")
                .and_then(|c| c.as_str())
                .map(String::from);
            return (guid, created);
        }
    }

    (None, None)
}

/// Extract `wefc://` model references from a manifest's WefcModel objects.
///
/// Scans the manifest `data` array for objects of type `WefcModel` and
/// returns their GUIDs formatted as `wefc://<guid>` references.
fn extract_model_refs(manifest: &serde_json::Value) -> Vec<serde_json::Value> {
    let empty = vec![];
    let data = manifest
        .get("data")
        .and_then(|d| d.as_array())
        .unwrap_or(&empty);

    data.iter()
        .filter(|obj| {
            obj.get("type")
                .and_then(|t| t.as_str())
                .map(|t| t == "WefcModel")
                .unwrap_or(false)
        })
        .filter_map(|obj| {
            obj.get("guid")
                .and_then(|g| g.as_str())
                .map(|guid| serde_json::json!(format!("wefc://{guid}")))
        })
        .collect()
}

/// Extract the CloudClient from state, returning 503 if not configured.
fn require_cloud(state: &AppState) -> AppResult<&Arc<CloudClient>> {
    state
        .cloud
        .as_ref()
        .ok_or_else(|| AppError::Internal("cloud storage not configured".to_string()))
}
