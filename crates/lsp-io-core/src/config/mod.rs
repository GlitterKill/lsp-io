use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_prefer_path")]
    pub prefer_path: bool,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
    #[serde(default)]
    pub overrides: Vec<ServerOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerOverride {
    pub id: String,
    #[serde(default)]
    pub binary_path: Option<PathBuf>,
    #[serde(default)]
    pub args: Vec<String>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            prefer_path: default_prefer_path(),
            timeout: default_timeout(),
            cache_dir: None,
            overrides: Vec::new(),
        }
    }
}

impl ProjectConfig {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(".lsp-io.toml");
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = std::fs::read_to_string(&path)?;
        Ok(toml::from_str(&raw)?)
    }

    pub fn override_for(&self, id: &str) -> Option<&ServerOverride> {
        self.overrides
            .iter()
            .find(|entry| entry.id.eq_ignore_ascii_case(id))
    }
}

fn default_prefer_path() -> bool {
    true
}

fn default_timeout() -> u64 {
    300
}
