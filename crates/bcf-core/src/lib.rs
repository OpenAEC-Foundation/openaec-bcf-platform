//! # bcf-core
//!
//! BCF 2.1 parsing and generation library.
//! Handles .bcfzip files, markup.bcf XML, and viewpoint.bcfv XML.
//! No web framework dependencies — pure data types and file operations.

pub mod error;
pub mod types;
pub mod xml_types;

pub mod bcfzip;
pub mod markup;
pub mod visinfo;
