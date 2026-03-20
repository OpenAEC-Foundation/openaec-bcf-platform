//! BCF-specific error types for XML parsing and ZIP handling.

/// Errors that can occur during BCF file operations.
#[derive(Debug, thiserror::Error)]
pub enum BcfError {
  #[error("XML parse error: {0}")]
  Xml(#[from] quick_xml::DeError),

  #[error("ZIP error: {0}")]
  Zip(#[from] zip::result::ZipError),

  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("missing required file: {0}")]
  MissingFile(String),

  #[error("invalid BCF data: {0}")]
  InvalidData(String),
}
