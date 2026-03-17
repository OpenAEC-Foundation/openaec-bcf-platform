//! Shared BCF 2.1 data types.
//!
//! These types represent the BCF data model and are used by both
//! the core library (XML serialization) and the server (API + DB).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// BCF project metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BcfProject {
  pub id: Uuid,
  pub name: String,
  #[serde(default)]
  pub description: String,
}

/// BCF topic (issue).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BcfTopic {
  pub guid: Uuid,
  pub title: String,
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub topic_type: String,
  #[serde(default = "default_status")]
  pub topic_status: String,
  #[serde(default = "default_priority")]
  pub priority: String,
  #[serde(default)]
  pub stage: String,
  #[serde(default)]
  pub labels: Vec<String>,
  pub due_date: Option<NaiveDate>,
  pub assigned_to: Option<String>,
  pub creation_author: Option<String>,
  pub modified_author: Option<String>,
  pub creation_date: Option<DateTime<Utc>>,
  pub modified_date: Option<DateTime<Utc>>,
  pub index: Option<i32>,
}

fn default_status() -> String {
  "Open".to_string()
}

fn default_priority() -> String {
  "Normal".to_string()
}

/// BCF comment on a topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BcfComment {
  pub guid: Uuid,
  pub comment: String,
  pub author: Option<String>,
  pub viewpoint_guid: Option<Uuid>,
  pub date: Option<DateTime<Utc>>,
  pub modified_date: Option<DateTime<Utc>>,
  pub modified_author: Option<String>,
}

/// Camera position for a viewpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
  #[serde(default = "default_camera_type")]
  pub camera_type: String,
  pub position: Point3D,
  pub direction: Point3D,
  pub up: Point3D,
  pub field_of_view: Option<f64>,
  pub aspect_ratio: Option<f64>,
}

fn default_camera_type() -> String {
  "perspective".to_string()
}

/// 3D point used in camera definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point3D {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

/// Component visibility and selection state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Components {
  #[serde(default)]
  pub visibility: ComponentVisibility,
  #[serde(default)]
  pub selection: Vec<ComponentRef>,
  #[serde(default)]
  pub coloring: Vec<ColoredComponents>,
}

/// Visibility settings for components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentVisibility {
  #[serde(default = "default_true")]
  pub default_visibility: bool,
  #[serde(default)]
  pub exceptions: Vec<ComponentRef>,
}

impl Default for ComponentVisibility {
  fn default() -> Self {
    Self {
      default_visibility: true,
      exceptions: Vec::new(),
    }
  }
}

fn default_true() -> bool {
  true
}

/// Reference to a component by IFC GlobalId.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRef {
  pub ifc_guid: String,
  #[serde(default)]
  pub originating_system: String,
  #[serde(default)]
  pub authoring_tool_id: String,
}

/// Group of components with a shared color.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColoredComponents {
  pub color: String,
  pub components: Vec<ComponentRef>,
}

/// BCF viewpoint (camera + snapshot + components).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BcfViewpoint {
  pub guid: Uuid,
  pub camera: Option<Camera>,
  pub components: Option<Components>,
  pub snapshot_data: Option<Vec<u8>>,
}

/// Project extensions — custom enums for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectExtensions {
  #[serde(default)]
  pub topic_types: Vec<String>,
  #[serde(default = "default_statuses")]
  pub topic_statuses: Vec<String>,
  #[serde(default = "default_priorities")]
  pub priorities: Vec<String>,
  #[serde(default)]
  pub labels: Vec<String>,
  #[serde(default)]
  pub stages: Vec<String>,
}

fn default_statuses() -> Vec<String> {
  vec!["Open".to_string(), "Closed".to_string()]
}

fn default_priorities() -> Vec<String> {
  vec!["Normal".to_string()]
}

impl Default for ProjectExtensions {
  fn default() -> Self {
    Self {
      topic_types: Vec::new(),
      topic_statuses: default_statuses(),
      priorities: default_priorities(),
      labels: Vec::new(),
      stages: Vec::new(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn topic_serializes_with_defaults() {
    let topic = BcfTopic {
      guid: Uuid::new_v4(),
      title: "Test issue".to_string(),
      description: String::new(),
      topic_type: String::new(),
      topic_status: "Open".to_string(),
      priority: "Normal".to_string(),
      stage: String::new(),
      labels: vec!["IDS".to_string()],
      due_date: None,
      assigned_to: None,
      creation_author: Some("test@example.com".to_string()),
      modified_author: None,
      creation_date: Some(Utc::now()),
      modified_date: None,
      index: Some(1),
    };

    let json = serde_json::to_string(&topic).unwrap();
    let parsed: BcfTopic = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.title, "Test issue");
    assert_eq!(parsed.labels, vec!["IDS"]);
  }

  #[test]
  fn camera_roundtrips_through_json() {
    let camera = Camera {
      camera_type: "perspective".to_string(),
      position: Point3D {
        x: 1.0,
        y: 2.0,
        z: 3.0,
      },
      direction: Point3D {
        x: 0.0,
        y: 0.0,
        z: -1.0,
      },
      up: Point3D {
        x: 0.0,
        y: 1.0,
        z: 0.0,
      },
      field_of_view: Some(60.0),
      aspect_ratio: Some(1.777),
    };

    let json = serde_json::to_string(&camera).unwrap();
    let parsed: Camera = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.position.x, 1.0);
    assert_eq!(parsed.field_of_view, Some(60.0));
  }

  #[test]
  fn extensions_have_sane_defaults() {
    let ext = ProjectExtensions::default();
    assert_eq!(ext.topic_statuses, vec!["Open", "Closed"]);
    assert_eq!(ext.priorities, vec!["Normal"]);
    assert!(ext.topic_types.is_empty());
  }
}
