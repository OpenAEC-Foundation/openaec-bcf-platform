//! Nextcloud WebDAV client for cloud file storage.
//!
//! Proxies file operations to Nextcloud via WebDAV over the internal
//! Docker network. Uses Basic auth with a service account.

use reqwest::Client;
use serde::Serialize;

use crate::error::{AppError, AppResult};

/// Tool-specific subdirectory within each project.
const TOOL_SLUG: &str = "bcf-platform";

/// Root folder on Nextcloud where all projects live.
const PROJECTS_ROOT: &str = "Projects";

/// A file entry returned from a directory listing.
#[derive(Debug, Serialize)]
pub struct CloudFile {
  pub name: String,
  pub size: u64,
  pub last_modified: String,
}

/// A project folder entry.
#[derive(Debug, Serialize)]
pub struct CloudProject {
  pub name: String,
}

/// Nextcloud WebDAV client.
#[derive(Debug, Clone)]
pub struct NextcloudClient {
  client: Client,
  webdav_root: String,
  username: String,
  password: String,
}

impl NextcloudClient {
  /// Create a new client from config values.
  pub fn new(base_url: &str, username: &str, password: &str) -> Self {
    let base = base_url.trim_end_matches('/');
    let encoded_user = urlencoding::encode(username);
    Self {
      client: Client::new(),
      webdav_root: format!("{base}/remote.php/dav/files/{encoded_user}"),
      username: username.to_string(),
      password: password.to_string(),
    }
  }

  /// Test if Nextcloud is reachable.
  pub async fn test_connection(&self) -> AppResult<bool> {
    let url = format!("{}/{PROJECTS_ROOT}/", self.webdav_root);
    let resp = self
      .client
      .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
      .basic_auth(&self.username, Some(&self.password))
      .header("Depth", "0")
      .send()
      .await
      .map_err(|e| AppError::Internal(format!("nextcloud unreachable: {e}")))?;
    Ok(resp.status().is_success() || resp.status().as_u16() == 207)
  }

  /// List all project folders under Projects/.
  pub async fn list_projects(&self) -> AppResult<Vec<CloudProject>> {
    let url = format!("{}/{PROJECTS_ROOT}/", self.webdav_root);
    let entries = self.propfind(&url).await?;
    let projects = entries
      .into_iter()
      .filter(|e| e.is_collection)
      .map(|e| CloudProject { name: e.name })
      .collect();
    Ok(projects)
  }

  /// List files in a project's tool subdirectory.
  pub async fn list_files(&self, project: &str) -> AppResult<Vec<CloudFile>> {
    let path = self.tool_path(project);
    let url = format!("{}/{path}/", self.webdav_root);
    let entries = self.propfind(&url).await;

    match entries {
      Ok(items) => Ok(
        items
          .into_iter()
          .filter(|e| !e.is_collection)
          .map(|e| CloudFile {
            name: e.name,
            size: e.size,
            last_modified: e.last_modified,
          })
          .collect(),
      ),
      Err(_) => Ok(vec![]), // Directory doesn't exist yet
    }
  }

  /// Download a file from the project's tool subdirectory.
  pub async fn download_file(&self, project: &str, filename: &str) -> AppResult<Vec<u8>> {
    let path = self.tool_path(project);
    let encoded_name = urlencoding::encode(filename);
    let url = format!("{}/{path}/{encoded_name}", self.webdav_root);

    let resp = self
      .client
      .get(&url)
      .basic_auth(&self.username, Some(&self.password))
      .send()
      .await
      .map_err(|e| AppError::Internal(format!("nextcloud download failed: {e}")))?;

    if resp.status().as_u16() == 404 {
      return Err(AppError::NotFound(format!("file not found: {filename}")));
    }
    if !resp.status().is_success() {
      return Err(AppError::Internal(format!(
        "nextcloud error: {}",
        resp.status()
      )));
    }

    resp
      .bytes()
      .await
      .map(|b| b.to_vec())
      .map_err(|e| AppError::Internal(format!("read error: {e}")))
  }

  /// Upload (create or overwrite) a file in the project's tool subdirectory.
  pub async fn upload_file(
    &self,
    project: &str,
    filename: &str,
    data: Vec<u8>,
  ) -> AppResult<()> {
    // Auto-create directory hierarchy
    self.ensure_tool_dir(project).await?;

    let path = self.tool_path(project);
    let encoded_name = urlencoding::encode(filename);
    let url = format!("{}/{path}/{encoded_name}", self.webdav_root);

    let resp = self
      .client
      .put(&url)
      .basic_auth(&self.username, Some(&self.password))
      .body(data)
      .send()
      .await
      .map_err(|e| AppError::Internal(format!("nextcloud upload failed: {e}")))?;

    if !resp.status().is_success() && resp.status().as_u16() != 201 && resp.status().as_u16() != 204
    {
      return Err(AppError::Internal(format!(
        "nextcloud upload error: {}",
        resp.status()
      )));
    }

    Ok(())
  }

  /// Delete a file from the project's tool subdirectory.
  pub async fn delete_file(&self, project: &str, filename: &str) -> AppResult<()> {
    let path = self.tool_path(project);
    let encoded_name = urlencoding::encode(filename);
    let url = format!("{}/{path}/{encoded_name}", self.webdav_root);

    let resp = self
      .client
      .delete(&url)
      .basic_auth(&self.username, Some(&self.password))
      .send()
      .await
      .map_err(|e| AppError::Internal(format!("nextcloud delete failed: {e}")))?;

    if resp.status().as_u16() == 404 {
      return Err(AppError::NotFound(format!("file not found: {filename}")));
    }

    Ok(())
  }

  /// Build the relative path to a project's tool subdirectory.
  fn tool_path(&self, project: &str) -> String {
    let safe_project = urlencoding::encode(project);
    format!("{PROJECTS_ROOT}/{safe_project}/99_overige_documenten/{TOOL_SLUG}")
  }

  /// Ensure the full directory hierarchy exists for a project's tool subdir.
  async fn ensure_tool_dir(&self, project: &str) -> AppResult<()> {
    let safe_project = urlencoding::encode(project);
    let segments = [
      PROJECTS_ROOT.to_string(),
      format!("{PROJECTS_ROOT}/{safe_project}"),
      format!("{PROJECTS_ROOT}/{safe_project}/99_overige_documenten"),
      format!("{PROJECTS_ROOT}/{safe_project}/99_overige_documenten/{TOOL_SLUG}"),
    ];

    for seg in &segments {
      let url = format!("{}/{seg}/", self.webdav_root);
      let resp = self
        .client
        .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), &url)
        .basic_auth(&self.username, Some(&self.password))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("mkcol failed: {e}")))?;

      let status = resp.status().as_u16();
      // 201 = created, 405 = already exists — both OK
      if status != 201 && status != 405 && !resp.status().is_success() {
        return Err(AppError::Internal(format!(
          "mkcol {seg} failed: {status}"
        )));
      }
    }

    Ok(())
  }

  /// PROPFIND a URL and parse the WebDAV multistatus response.
  async fn propfind(&self, url: &str) -> AppResult<Vec<DavEntry>> {
    let resp = self
      .client
      .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), url)
      .basic_auth(&self.username, Some(&self.password))
      .header("Depth", "1")
      .send()
      .await
      .map_err(|e| AppError::Internal(format!("nextcloud unreachable: {e}")))?;

    if resp.status().as_u16() == 404 {
      return Err(AppError::NotFound("path not found on nextcloud".to_string()));
    }

    let body = resp
      .text()
      .await
      .map_err(|e| AppError::Internal(format!("read error: {e}")))?;

    parse_propfind_xml(&body)
  }
}

/// A parsed entry from a WebDAV PROPFIND response.
#[derive(Debug)]
struct DavEntry {
  name: String,
  is_collection: bool,
  size: u64,
  last_modified: String,
}

/// Parse WebDAV multistatus XML using quick-xml.
fn parse_propfind_xml(xml: &str) -> AppResult<Vec<DavEntry>> {
  use quick_xml::events::Event;
  use quick_xml::Reader;

  let mut reader = Reader::from_str(xml);
  let mut entries: Vec<DavEntry> = Vec::new();

  // Track state while parsing
  let mut in_response = false;
  let mut in_href = false;
  let mut in_displayname = false;
  let mut in_contentlength = false;
  let mut in_lastmodified = false;
  let mut in_resourcetype = false;

  let mut current_href = String::new();
  let mut current_displayname = String::new();
  let mut current_size: u64 = 0;
  let mut current_lastmod = String::new();
  let mut current_is_collection = false;

  let mut buf = Vec::new();

  loop {
    match reader.read_event_into(&mut buf) {
      Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
        let qname = e.name();
        let local = local_name(qname.as_ref());
        match local {
          "response" => {
            in_response = true;
            current_href.clear();
            current_displayname.clear();
            current_size = 0;
            current_lastmod.clear();
            current_is_collection = false;
          }
          "href" if in_response => in_href = true,
          "displayname" if in_response => in_displayname = true,
          "getcontentlength" if in_response => in_contentlength = true,
          "getlastmodified" if in_response => in_lastmodified = true,
          "resourcetype" if in_response => in_resourcetype = true,
          "collection" if in_resourcetype => current_is_collection = true,
          _ => {}
        }
      }
      Ok(Event::End(ref e)) => {
        let qname = e.name();
        let local = local_name(qname.as_ref());
        match local {
          "response" => {
            in_response = false;
            // Derive name from href if displayname is empty
            let name = if current_displayname.is_empty() {
              name_from_href(&current_href)
            } else {
              current_displayname.clone()
            };

            if !name.is_empty() {
              entries.push(DavEntry {
                name,
                is_collection: current_is_collection,
                size: current_size,
                last_modified: current_lastmod.clone(),
              });
            }
          }
          "href" => in_href = false,
          "displayname" => in_displayname = false,
          "getcontentlength" => in_contentlength = false,
          "getlastmodified" => in_lastmodified = false,
          "resourcetype" => in_resourcetype = false,
          _ => {}
        }
      }
      Ok(Event::Text(e)) => {
        if let Ok(text) = e.unescape() {
          if in_href {
            current_href.push_str(&text);
          } else if in_displayname {
            current_displayname.push_str(&text);
          } else if in_contentlength {
            current_size = text.parse().unwrap_or(0);
          } else if in_lastmodified {
            current_lastmod.push_str(&text);
          }
        }
      }
      Ok(Event::Eof) => break,
      Err(e) => {
        return Err(AppError::Internal(format!("XML parse error: {e}")));
      }
      _ => {}
    }
    buf.clear();
  }

  // Skip the first entry (it's the directory itself)
  if !entries.is_empty() {
    entries.remove(0);
  }

  Ok(entries)
}

/// Extract local name from a possibly namespaced XML tag (e.g. `d:href` → `href`).
fn local_name(raw: &[u8]) -> &str {
  let s = std::str::from_utf8(raw).unwrap_or("");
  s.rsplit_once(':').map_or(s, |(_, local)| local)
}

/// Extract a display name from a WebDAV href path.
fn name_from_href(href: &str) -> String {
  let trimmed = href.trim_end_matches('/');
  let segment = trimmed.rsplit('/').next().unwrap_or("");
  urlencoding::decode(segment)
    .map(|s| s.into_owned())
    .unwrap_or_else(|_| segment.to_string())
}
