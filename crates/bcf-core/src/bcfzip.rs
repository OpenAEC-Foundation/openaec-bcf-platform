//! BCF ZIP archive reading and writing.
//!
//! Handles complete `.bcfzip` files containing multiple topic folders,
//! each with `markup.bcf`, `viewpoint.bcfv`, and `snapshot.png`.

use crate::error::BcfError;
use crate::types::{BcfComment, BcfTopic, BcfViewpoint};

/// Parsed BCF archive containing all topics.
#[derive(Debug, Clone)]
pub struct BcfArchive {
  pub version: String,
  pub topics: Vec<BcfTopicFolder>,
}

/// A single topic folder from a BCF archive.
#[derive(Debug, Clone)]
pub struct BcfTopicFolder {
  pub topic: BcfTopic,
  pub comments: Vec<BcfComment>,
  pub viewpoints: Vec<BcfViewpoint>,
}

/// Read a `.bcfzip` file from bytes into a [`BcfArchive`].
pub fn read_bcfzip(data: &[u8]) -> Result<BcfArchive, BcfError> {
  use std::io::{Cursor, Read};

  use crate::markup;
  use crate::visinfo;
  use crate::xml_types::XmlVersion;

  let cursor = Cursor::new(data);
  let mut archive = zip::ZipArchive::new(cursor)?;

  // Read version file
  let version = if let Ok(mut f) = archive.by_name("bcf.version") {
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    let ver: XmlVersion = quick_xml::de::from_str(&buf)
      .map_err(|e| BcfError::InvalidData(format!("invalid bcf.version: {e}")))?;
    ver.version_id
  } else {
    "2.1".to_string()
  };

  // Collect topic folder names (directories containing markup.bcf)
  let mut topic_folders: Vec<String> = Vec::new();
  for i in 0..archive.len() {
    let entry = archive.by_index(i)?;
    let name = entry.name().to_string();
    if name.ends_with("/markup.bcf") {
      if let Some(folder) = name.strip_suffix("/markup.bcf") {
        topic_folders.push(folder.to_string());
      }
    }
  }

  let mut topics = Vec::new();

  for folder in &topic_folders {
    // Parse markup.bcf
    let markup_path = format!("{folder}/markup.bcf");
    let mut markup_data = Vec::new();
    archive.by_name(&markup_path)?.read_to_end(&mut markup_data)?;
    let (topic, comments, vp_refs) = markup::parse_markup(&markup_data)?;

    // Parse viewpoints (read each file separately to avoid borrow conflicts)
    let mut viewpoints = Vec::new();
    for vp_ref in &vp_refs {
      let bcfv_path = format!("{folder}/{}", vp_ref.viewpoint_file);
      let bcfv_data = {
        let mut f = match archive.by_name(&bcfv_path) {
          Ok(f) => f,
          Err(_) => continue,
        };
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        buf
      };
      let mut vp = visinfo::parse_visinfo(&bcfv_data)?;

      // Load snapshot if referenced
      if let Some(ref snap_file) = vp_ref.snapshot_file {
        let snap_path = format!("{folder}/{snap_file}");
        if let Ok(mut sf) = archive.by_name(&snap_path) {
          let mut snap_data = Vec::new();
          sf.read_to_end(&mut snap_data)?;
          vp.snapshot_data = Some(snap_data);
        }
      }

      viewpoints.push(vp);
    }

    topics.push(BcfTopicFolder {
      topic,
      comments,
      viewpoints,
    });
  }

  Ok(BcfArchive { version, topics })
}

/// Write a [`BcfArchive`] to `.bcfzip` bytes.
pub fn write_bcfzip(archive: &BcfArchive) -> Result<Vec<u8>, BcfError> {
  use std::io::{Cursor, Write};

  use crate::markup::{self, ViewpointRef};
  use crate::visinfo;
  use crate::xml_types::XmlVersion;

  let buf = Cursor::new(Vec::new());
  let mut zip = zip::ZipWriter::new(buf);
  let options = zip::write::SimpleFileOptions::default()
    .compression_method(zip::CompressionMethod::Deflated);

  // Write bcf.version
  let version_xml = quick_xml::se::to_string(&XmlVersion {
    version_id: archive.version.clone(),
  })
  .map_err(|e| BcfError::InvalidData(format!("version XML failed: {e}")))?;

  zip.start_file("bcf.version", options)?;
  zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
  zip.write_all(version_xml.as_bytes())?;

  for folder in &archive.topics {
    let folder_name = folder.topic.guid.to_string();

    // Build viewpoint references for markup
    let vp_refs: Vec<ViewpointRef> = folder
      .viewpoints
      .iter()
      .map(|vp| ViewpointRef {
        guid: vp.guid,
        viewpoint_file: format!("{}.bcfv", vp.guid),
        snapshot_file: vp.snapshot_data.as_ref().map(|_| "snapshot.png".to_string()),
      })
      .collect();

    // Write markup.bcf
    let markup_xml = markup::generate_markup(&folder.topic, &folder.comments, &vp_refs)?;
    zip.start_file(format!("{folder_name}/markup.bcf"), options)?;
    zip.write_all(&markup_xml)?;

    // Write viewpoint files + snapshots
    for vp in &folder.viewpoints {
      let bcfv_data = visinfo::generate_visinfo(vp)?;
      zip.start_file(format!("{folder_name}/{}.bcfv", vp.guid), options)?;
      zip.write_all(&bcfv_data)?;

      if let Some(ref snap) = vp.snapshot_data {
        zip.start_file(format!("{folder_name}/snapshot.png"), options)?;
        zip.write_all(snap)?;
      }
    }
  }

  let cursor = zip.finish()?;
  Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::types::{Camera, Point3D};
  use uuid::Uuid;

  fn sample_archive() -> BcfArchive {
    let topic = BcfTopic {
      guid: Uuid::parse_str("d1d2d3d4-d5d6-d7d8-d9da-dbdcdddedfe0").unwrap(),
      title: "Test issue".to_string(),
      description: "A test".to_string(),
      topic_type: "Error".to_string(),
      topic_status: "Open".to_string(),
      priority: "Normal".to_string(),
      stage: String::new(),
      labels: vec!["Architecture".to_string()],
      due_date: None,
      assigned_to: None,
      creation_author: Some("test@example.com".to_string()),
      modified_author: None,
      creation_date: None,
      modified_date: None,
      index: Some(1),
    };

    let comment = BcfComment {
      guid: Uuid::parse_str("e1e2e3e4-e5e6-e7e8-e9ea-ebecedeeeff0").unwrap(),
      comment: "Fix this".to_string(),
      author: Some("reviewer@example.com".to_string()),
      viewpoint_guid: None,
      date: None,
      modified_date: None,
      modified_author: None,
    };

    let viewpoint = BcfViewpoint {
      guid: Uuid::parse_str("f1f2f3f4-f5f6-f7f8-f9fa-fbfcfdfeff00").unwrap(),
      camera: Some(Camera {
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
      }),
      components: None,
      snapshot_data: Some(b"fake-png-data".to_vec()),
    };

    BcfArchive {
      version: "2.1".to_string(),
      topics: vec![BcfTopicFolder {
        topic,
        comments: vec![comment],
        viewpoints: vec![viewpoint],
      }],
    }
  }

  #[test]
  fn roundtrip_bcfzip() {
    let archive = sample_archive();
    let zip_bytes = write_bcfzip(&archive).unwrap();
    let parsed = read_bcfzip(&zip_bytes).unwrap();

    assert_eq!(parsed.version, "2.1");
    assert_eq!(parsed.topics.len(), 1);

    let folder = &parsed.topics[0];
    assert_eq!(folder.topic.title, "Test issue");
    assert_eq!(folder.topic.topic_type, "Error");
    assert_eq!(folder.topic.labels, vec!["Architecture"]);
    assert_eq!(folder.comments.len(), 1);
    assert_eq!(folder.comments[0].comment, "Fix this");
    assert_eq!(folder.viewpoints.len(), 1);

    let vp = &folder.viewpoints[0];
    let cam = vp.camera.as_ref().unwrap();
    assert_eq!(cam.camera_type, "perspective");
    assert_eq!(cam.position.x, 1.0);
    assert_eq!(vp.snapshot_data.as_ref().unwrap(), b"fake-png-data");
  }

  #[test]
  fn empty_archive_roundtrips() {
    let archive = BcfArchive {
      version: "2.1".to_string(),
      topics: vec![],
    };
    let zip_bytes = write_bcfzip(&archive).unwrap();
    let parsed = read_bcfzip(&zip_bytes).unwrap();
    assert!(parsed.topics.is_empty());
  }
}
