use std::path::{Path, PathBuf};

use mad_core::{evaluation::sample_vendors, EvaluationWorkspace, PolicyBundle};
use tokio::sync::RwLock;

pub struct WorkspaceStore {
    path: PathBuf,
    inner: RwLock<EvaluationWorkspace>,
}

impl WorkspaceStore {
    pub async fn load(policy: &PolicyBundle, path: PathBuf) -> Self {
        let workspace = if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
                    tracing::warn!("failed to parse workspace, using defaults: {e}");
                    default_workspace(policy)
                }),
                Err(e) => {
                    tracing::warn!("failed to read workspace: {e}");
                    default_workspace(policy)
                }
            }
        } else {
            default_workspace(policy)
        };

        let store = Self {
            path,
            inner: RwLock::new(workspace),
        };
        let _ = store.persist().await;
        store
    }

    pub async fn get(&self) -> EvaluationWorkspace {
        self.inner.read().await.clone()
    }

    pub async fn replace(&self, workspace: EvaluationWorkspace) -> Result<(), std::io::Error> {
        *self.inner.write().await = workspace;
        self.persist().await
    }

    pub async fn update<F>(&self, f: F) -> Result<EvaluationWorkspace, std::io::Error>
    where
        F: FnOnce(&mut EvaluationWorkspace),
    {
        {
            let mut guard = self.inner.write().await;
            f(&mut guard);
        }
        self.persist().await?;
        Ok(self.get().await)
    }

    async fn persist(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let guard = self.inner.read().await;
        let json = serde_json::to_string_pretty(&*guard).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;
        std::fs::write(&self.path, json)
    }
}

fn default_workspace(policy: &PolicyBundle) -> EvaluationWorkspace {
    let mut workspace = EvaluationWorkspace::from_policy(policy);
    for (vendor, assessment) in sample_vendors() {
        workspace.vendors.push(vendor);
        workspace
            .assessments
            .insert(assessment.vendor_id.0.clone(), assessment);
    }
    workspace
}

pub fn workspace_path(data_dir: &Path) -> PathBuf {
    data_dir.join("workspace.json")
}
