//! XML serde structs for BCF 2.1 markup.bcf and viewpoint.bcfv files.
//!
//! These types map directly to the BCF XML schema and are separate from
//! the JSON/API types in [`crate::types`]. Conversion happens via `From` impls.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// bcf.version
// ---------------------------------------------------------------------------

/// Root element of `bcf.version`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Version")]
pub struct XmlVersion {
  #[serde(rename = "@VersionId")]
  pub version_id: String,
}

// ---------------------------------------------------------------------------
// markup.bcf
// ---------------------------------------------------------------------------

/// Root element of `markup.bcf`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Markup")]
pub struct XmlMarkup {
  #[serde(rename = "Topic")]
  pub topic: XmlTopic,

  #[serde(rename = "Comment", default)]
  pub comments: Vec<XmlComment>,

  #[serde(rename = "Viewpoints", default)]
  pub viewpoints: Vec<XmlViewpointRef>,
}

/// `<Topic>` element within markup.bcf.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Topic")]
pub struct XmlTopic {
  #[serde(rename = "@Guid")]
  pub guid: String,

  #[serde(rename = "@TopicType", default)]
  pub topic_type: String,

  #[serde(rename = "@TopicStatus", default)]
  pub topic_status: String,

  #[serde(rename = "Title")]
  pub title: String,

  #[serde(rename = "Description", default)]
  pub description: String,

  #[serde(rename = "Priority", default)]
  pub priority: String,

  #[serde(rename = "Index", skip_serializing_if = "Option::is_none")]
  pub index: Option<i32>,

  #[serde(rename = "CreationDate", skip_serializing_if = "Option::is_none")]
  pub creation_date: Option<String>,

  #[serde(rename = "CreationAuthor", skip_serializing_if = "Option::is_none")]
  pub creation_author: Option<String>,

  #[serde(rename = "ModifiedDate", skip_serializing_if = "Option::is_none")]
  pub modified_date: Option<String>,

  #[serde(rename = "ModifiedAuthor", skip_serializing_if = "Option::is_none")]
  pub modified_author: Option<String>,

  #[serde(rename = "AssignedTo", skip_serializing_if = "Option::is_none")]
  pub assigned_to: Option<String>,

  #[serde(rename = "Stage", skip_serializing_if = "Option::is_none")]
  pub stage: Option<String>,

  #[serde(rename = "DueDate", skip_serializing_if = "Option::is_none")]
  pub due_date: Option<String>,

  #[serde(rename = "Labels", skip_serializing_if = "Option::is_none")]
  pub labels: Option<XmlLabels>,
}

/// Container for `<Label>` elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlLabels {
  #[serde(rename = "Label", default)]
  pub labels: Vec<String>,
}

/// `<Comment>` element within markup.bcf.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Comment")]
pub struct XmlComment {
  #[serde(rename = "@Guid")]
  pub guid: String,

  #[serde(rename = "Date", skip_serializing_if = "Option::is_none")]
  pub date: Option<String>,

  #[serde(rename = "Author", skip_serializing_if = "Option::is_none")]
  pub author: Option<String>,

  #[serde(rename = "Comment")]
  pub comment: String,

  #[serde(rename = "Viewpoint", skip_serializing_if = "Option::is_none")]
  pub viewpoint: Option<XmlCommentViewpointRef>,

  #[serde(rename = "ModifiedDate", skip_serializing_if = "Option::is_none")]
  pub modified_date: Option<String>,

  #[serde(rename = "ModifiedAuthor", skip_serializing_if = "Option::is_none")]
  pub modified_author: Option<String>,
}

/// Viewpoint reference inside a `<Comment>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlCommentViewpointRef {
  #[serde(rename = "@Guid")]
  pub guid: String,
}

/// `<Viewpoints>` element in markup — references a viewpoint file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Viewpoints")]
pub struct XmlViewpointRef {
  #[serde(rename = "@Guid")]
  pub guid: String,

  #[serde(rename = "Viewpoint")]
  pub viewpoint: String,

  #[serde(rename = "Snapshot", skip_serializing_if = "Option::is_none")]
  pub snapshot: Option<String>,
}

// ---------------------------------------------------------------------------
// viewpoint.bcfv (VisualizationInfo)
// ---------------------------------------------------------------------------

/// Root element of a `viewpoint.bcfv` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "VisualizationInfo")]
pub struct XmlVisualizationInfo {
  #[serde(rename = "@Guid")]
  pub guid: String,

  #[serde(rename = "PerspectiveCamera", skip_serializing_if = "Option::is_none")]
  pub perspective_camera: Option<XmlPerspectiveCamera>,

  #[serde(rename = "OrthogonalCamera", skip_serializing_if = "Option::is_none")]
  pub orthogonal_camera: Option<XmlOrthogonalCamera>,

  #[serde(rename = "Components", skip_serializing_if = "Option::is_none")]
  pub components: Option<XmlComponents>,
}

/// `<PerspectiveCamera>` element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlPerspectiveCamera {
  #[serde(rename = "CameraViewPoint")]
  pub camera_view_point: XmlPoint3D,

  #[serde(rename = "CameraDirection")]
  pub camera_direction: XmlPoint3D,

  #[serde(rename = "CameraUpVector")]
  pub camera_up_vector: XmlPoint3D,

  #[serde(rename = "FieldOfView")]
  pub field_of_view: f64,

  #[serde(rename = "AspectRatio", default = "default_aspect_ratio")]
  pub aspect_ratio: f64,
}

fn default_aspect_ratio() -> f64 {
  1.0
}

/// `<OrthogonalCamera>` element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlOrthogonalCamera {
  #[serde(rename = "CameraViewPoint")]
  pub camera_view_point: XmlPoint3D,

  #[serde(rename = "CameraDirection")]
  pub camera_direction: XmlPoint3D,

  #[serde(rename = "CameraUpVector")]
  pub camera_up_vector: XmlPoint3D,

  #[serde(rename = "ViewToWorldScale")]
  pub view_to_world_scale: f64,

  #[serde(rename = "AspectRatio", default = "default_aspect_ratio")]
  pub aspect_ratio: f64,
}

/// 3D point with `<X>`, `<Y>`, `<Z>` child elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlPoint3D {
  #[serde(rename = "X")]
  pub x: f64,
  #[serde(rename = "Y")]
  pub y: f64,
  #[serde(rename = "Z")]
  pub z: f64,
}

/// `<Components>` element in VisualizationInfo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlComponents {
  #[serde(rename = "Visibility", skip_serializing_if = "Option::is_none")]
  pub visibility: Option<XmlVisibility>,

  #[serde(rename = "Selection", skip_serializing_if = "Option::is_none")]
  pub selection: Option<XmlSelection>,

  #[serde(rename = "Coloring", skip_serializing_if = "Option::is_none")]
  pub coloring: Option<XmlColoring>,
}

/// `<Visibility>` element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlVisibility {
  #[serde(rename = "@DefaultVisibility", default = "default_true")]
  pub default_visibility: bool,

  #[serde(rename = "Exceptions", skip_serializing_if = "Option::is_none")]
  pub exceptions: Option<XmlExceptions>,
}

fn default_true() -> bool {
  true
}

/// Container for visibility exception components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlExceptions {
  #[serde(rename = "Component", default)]
  pub components: Vec<XmlComponent>,
}

/// Container for selected components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlSelection {
  #[serde(rename = "Component", default)]
  pub components: Vec<XmlComponent>,
}

/// `<Coloring>` container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlColoring {
  #[serde(rename = "Color", default)]
  pub colors: Vec<XmlColor>,
}

/// `<Color>` element with attribute and component children.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlColor {
  #[serde(rename = "@Color")]
  pub color: String,

  #[serde(rename = "Component", default)]
  pub components: Vec<XmlComponent>,
}

/// `<Component>` element referencing an IFC entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlComponent {
  #[serde(rename = "@IfcGuid")]
  pub ifc_guid: String,

  #[serde(rename = "OriginatingSystem", default)]
  pub originating_system: String,

  #[serde(rename = "AuthoringToolId", default)]
  pub authoring_tool_id: String,
}

// ---------------------------------------------------------------------------
// From impls: XML types ↔ domain types
// ---------------------------------------------------------------------------

use crate::types::{
  Camera, ColoredComponents, ComponentRef, ComponentVisibility, Components,
  Point3D,
};

impl From<XmlPoint3D> for Point3D {
  fn from(p: XmlPoint3D) -> Self {
    Self {
      x: p.x,
      y: p.y,
      z: p.z,
    }
  }
}

impl From<&Point3D> for XmlPoint3D {
  fn from(p: &Point3D) -> Self {
    Self {
      x: p.x,
      y: p.y,
      z: p.z,
    }
  }
}

impl From<XmlComponent> for ComponentRef {
  fn from(c: XmlComponent) -> Self {
    Self {
      ifc_guid: c.ifc_guid,
      originating_system: c.originating_system,
      authoring_tool_id: c.authoring_tool_id,
    }
  }
}

impl From<&ComponentRef> for XmlComponent {
  fn from(c: &ComponentRef) -> Self {
    Self {
      ifc_guid: c.ifc_guid.clone(),
      originating_system: c.originating_system.clone(),
      authoring_tool_id: c.authoring_tool_id.clone(),
    }
  }
}

impl From<XmlPerspectiveCamera> for Camera {
  fn from(c: XmlPerspectiveCamera) -> Self {
    Self {
      camera_type: "perspective".to_string(),
      position: c.camera_view_point.into(),
      direction: c.camera_direction.into(),
      up: c.camera_up_vector.into(),
      field_of_view: Some(c.field_of_view),
      aspect_ratio: Some(c.aspect_ratio),
    }
  }
}

impl From<XmlOrthogonalCamera> for Camera {
  fn from(c: XmlOrthogonalCamera) -> Self {
    Self {
      camera_type: "orthogonal".to_string(),
      position: c.camera_view_point.into(),
      direction: c.camera_direction.into(),
      up: c.camera_up_vector.into(),
      field_of_view: Some(c.view_to_world_scale),
      aspect_ratio: Some(c.aspect_ratio),
    }
  }
}

impl From<XmlComponents> for Components {
  fn from(c: XmlComponents) -> Self {
    let visibility = c
      .visibility
      .map(|v| ComponentVisibility {
        default_visibility: v.default_visibility,
        exceptions: v
          .exceptions
          .map(|e| e.components.into_iter().map(Into::into).collect())
          .unwrap_or_default(),
      })
      .unwrap_or_default();

    let selection = c
      .selection
      .map(|s| s.components.into_iter().map(Into::into).collect())
      .unwrap_or_default();

    let coloring = c
      .coloring
      .map(|col| {
        col
          .colors
          .into_iter()
          .map(|color| ColoredComponents {
            color: color.color,
            components: color.components.into_iter().map(Into::into).collect(),
          })
          .collect()
      })
      .unwrap_or_default();

    Self {
      visibility,
      selection,
      coloring,
    }
  }
}

impl From<&Components> for XmlComponents {
  fn from(c: &Components) -> Self {
    let visibility = Some(XmlVisibility {
      default_visibility: c.visibility.default_visibility,
      exceptions: if c.visibility.exceptions.is_empty() {
        None
      } else {
        Some(XmlExceptions {
          components: c.visibility.exceptions.iter().map(Into::into).collect(),
        })
      },
    });

    let selection = if c.selection.is_empty() {
      None
    } else {
      Some(XmlSelection {
        components: c.selection.iter().map(Into::into).collect(),
      })
    };

    let coloring = if c.coloring.is_empty() {
      None
    } else {
      Some(XmlColoring {
        colors: c
          .coloring
          .iter()
          .map(|cc| XmlColor {
            color: cc.color.clone(),
            components: cc.components.iter().map(Into::into).collect(),
          })
          .collect(),
      })
    };

    Self {
      visibility,
      selection,
      coloring,
    }
  }
}
