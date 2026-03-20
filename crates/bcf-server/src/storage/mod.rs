//! Snapshot and file storage for viewpoint images.
//!
//! Stores viewpoint snapshots on the local filesystem.
//! Path layout: `{base_path}/{topic_id}/{viewpoint_id}.png`

use std::path::PathBuf;

use uuid::Uuid;

/// Local filesystem storage for viewpoint snapshot images.
#[derive(Debug, Clone)]
pub struct SnapshotStorage {
  base_path: PathBuf,
}

impl SnapshotStorage {
  /// Create a new storage instance rooted at `base_path`.
  pub fn new(base_path: impl Into<PathBuf>) -> Self {
    Self {
      base_path: base_path.into(),
    }
  }

  /// Save snapshot bytes and return the relative storage path.
  pub async fn save(
    &self,
    topic_id: Uuid,
    viewpoint_id: Uuid,
    data: &[u8],
  ) -> Result<String, std::io::Error> {
    let dir = self.base_path.join(topic_id.to_string());
    tokio::fs::create_dir_all(&dir).await?;

    let filename = format!("{viewpoint_id}.png");
    let full_path = dir.join(&filename);
    tokio::fs::write(&full_path, data).await?;

    Ok(format!("{topic_id}/{filename}"))
  }

  /// Load snapshot bytes from a relative storage path.
  pub async fn load(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
    let full_path = self.base_path.join(path);
    tokio::fs::read(&full_path).await
  }

  /// Delete a snapshot file by relative path.
  pub async fn delete(&self, path: &str) -> Result<(), std::io::Error> {
    let full_path = self.base_path.join(path);
    match tokio::fs::remove_file(&full_path).await {
      Ok(()) => Ok(()),
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
      Err(e) => Err(e),
    }
  }
}
