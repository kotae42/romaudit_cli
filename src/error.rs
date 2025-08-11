// src/error.rs - Error handling module

use std::error::Error;
use std::fmt;
use std::io;
use serde_json;
use quick_xml;

#[derive(Debug)]
#[allow(dead_code)]
pub enum RomAuditError {
    Io(io::Error),
    Json(serde_json::Error),
    Xml(quick_xml::Error),
    NoDatFile,
    InvalidPath(String),
    ParseError(String),
    ConfigError(String),
}

impl fmt::Display for RomAuditError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RomAuditError::Io(e) => write!(f, "IO error: {}", e),
            RomAuditError::Json(e) => write!(f, "JSON error: {}", e),
            RomAuditError::Xml(e) => write!(f, "XML error: {}", e),
            RomAuditError::NoDatFile => write!(f, "No .dat or .xml file found in current directory"),
            RomAuditError::InvalidPath(p) => write!(f, "Invalid path: {}", p),
            RomAuditError::ParseError(e) => write!(f, "Parse error: {}", e),
            RomAuditError::ConfigError(e) => write!(f, "Configuration error: {}", e),
        }
    }
}

impl Error for RomAuditError {}

impl From<io::Error> for RomAuditError {
    fn from(error: io::Error) -> Self {
        RomAuditError::Io(error)
    }
}

impl From<serde_json::Error> for RomAuditError {
    fn from(error: serde_json::Error) -> Self {
        RomAuditError::Json(error)
    }
}

impl From<quick_xml::Error> for RomAuditError {
    fn from(error: quick_xml::Error) -> Self {
        RomAuditError::Xml(error)
    }
}

pub type Result<T> = std::result::Result<T, RomAuditError>;