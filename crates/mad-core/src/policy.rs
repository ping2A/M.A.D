use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{MadError, MadResult};
use crate::pillar::Pillar;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyManifest {
    pub version: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyFile {
    pub manifest: PolicyManifest,
    pub pillars: Vec<Pillar>,
}

#[derive(Debug, Clone, Default)]
pub struct PolicyBundle {
    pub pillars: Vec<Pillar>,
    pub source_paths: Vec<PathBuf>,
}

impl PolicyBundle {
    pub fn load_dir(dir: impl AsRef<Path>) -> MadResult<Self> {
        let dir = dir.as_ref();
        let mut bundle = Self::default();

        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| MadError::io(dir, e))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
            })
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let policy = Self::load_file(&path)?;
            bundle.pillars.extend(policy.pillars);
            bundle.source_paths.push(path);
        }

        if bundle.pillars.is_empty() {
            return Err(MadError::EmptyBundle);
        }

        Ok(bundle)
    }

    pub fn load_file(path: impl AsRef<Path>) -> MadResult<PolicyFile> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path).map_err(|e| MadError::io(path, e))?;
        serde_yaml::from_str(&contents).map_err(|e| MadError::parse(path, e))
    }

    pub fn pillar_count(&self) -> usize {
        self.pillars.len()
    }

    pub fn total_requirements(&self) -> usize {
        self.pillars.iter().map(|p| p.requirement_count()).sum()
    }

    pub fn critical_requirements(&self) -> usize {
        self.pillars.iter().map(|p| p.critical_count()).sum()
    }
}
