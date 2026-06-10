use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MadError {
    #[error("failed to read policy file {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse policy file {path}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_yaml::Error,
    },

    #[error("unknown pillar: {0}")]
    UnknownPillar(String),

    #[error("vendor not found: {0}")]
    VendorNotFound(String),

    #[error("policy bundle is empty")]
    EmptyBundle,
}

impl MadError {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn parse(path: impl Into<PathBuf>, source: serde_yaml::Error) -> Self {
        Self::Parse {
            path: path.into(),
            source,
        }
    }
}

pub type MadResult<T> = Result<T, MadError>;
