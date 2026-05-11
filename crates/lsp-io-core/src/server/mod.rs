pub mod github_release;
mod github_specs;
pub mod install;
pub mod registry;

use crate::config::ProjectConfig;
use crate::language::LanguageKind;
pub use install::{
    InstallOutcome, clean_managed_cache_with_options, install_server, install_server_with_options,
    managed_binary_path, managed_binary_path_with_options, remove_server,
    remove_server_with_options,
};
pub use registry::{Footprint, InstallMethod, Maturity, REGISTRY, ServerEntry};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 300;

#[derive(Debug, Clone, Serialize)]
pub struct ServerStatusInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub language_display: String,
    pub language_category: String,
    pub language_category_display: String,
    pub version: String,
    pub binary_name: String,
    pub install_method: String,
    pub installed: bool,
    pub install_state: String,
    pub installed_path: Option<String>,
    pub can_install: bool,
    pub can_remove: bool,
    pub footprint: String,
    pub maturity: String,
    pub source_url: String,
    pub rationale: String,
    pub manual_instructions: Option<String>,
    pub install_warning: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub cache_dir: PathBuf,
    pub prefer_path: bool,
    pub timeout: Duration,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            cache_dir: default_cache_dir(),
            prefer_path: true,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }
}

impl ServerOptions {
    pub fn from_config(root: &Path, config: &ProjectConfig) -> Self {
        let cache_dir = config
            .cache_dir
            .as_ref()
            .map(|path| {
                if path.is_absolute() {
                    path.clone()
                } else {
                    root.join(path)
                }
            })
            .unwrap_or_else(default_cache_dir);

        Self {
            cache_dir,
            prefer_path: config.prefer_path,
            timeout: Duration::from_secs(config.timeout.max(1)),
        }
    }
}

pub fn cache_dir() -> PathBuf {
    default_cache_dir()
}

fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("lsp-io")
        .join("servers")
}

pub fn all_status() -> Vec<ServerStatusInfo> {
    all_status_with_options(&ServerOptions::default())
}

pub fn all_status_with_options(options: &ServerOptions) -> Vec<ServerStatusInfo> {
    REGISTRY
        .all()
        .iter()
        .map(|entry| status_for_entry_with_options(entry, options))
        .collect()
}

pub fn all_status_with_config(root: &Path, config: &ProjectConfig) -> Vec<ServerStatusInfo> {
    let options = ServerOptions::from_config(root, config);
    all_status_with_options(&options)
}

pub fn status_for_languages(languages: &[LanguageKind]) -> Vec<ServerStatusInfo> {
    status_for_languages_with_options(languages, &ServerOptions::default())
}

pub fn status_for_languages_with_options(
    languages: &[LanguageKind],
    options: &ServerOptions,
) -> Vec<ServerStatusInfo> {
    let mut seen = std::collections::HashSet::new();
    let mut statuses = Vec::new();

    for language in languages {
        for entry in REGISTRY.for_language(*language) {
            if seen.insert(entry.id) {
                statuses.push(status_for_entry_with_options(entry, options));
            }
        }
    }

    statuses
}

pub fn status_for_entry(entry: &ServerEntry) -> ServerStatusInfo {
    status_for_entry_with_options(entry, &ServerOptions::default())
}

pub fn status_for_entry_with_options(
    entry: &ServerEntry,
    options: &ServerOptions,
) -> ServerStatusInfo {
    let managed_path = install::managed_install_path(entry, options);
    let has_managed_install = managed_path.is_some();
    let system_path = find_on_path(entry);

    let (installed_path, state) = if options.prefer_path {
        if let Some(path) = system_path {
            (Some(path), "system")
        } else if let Some(path) = managed_path {
            (Some(path), "managed")
        } else {
            (None, "missing")
        }
    } else if let Some(path) = managed_path {
        (Some(path), "managed")
    } else if let Some(path) = system_path {
        (Some(path), "system")
    } else {
        (None, "missing")
    };

    ServerStatusInfo {
        id: entry.id.to_string(),
        name: entry.name.to_string(),
        language: entry.language.name().to_string(),
        language_display: entry.language.display_name().to_string(),
        language_category: entry.language.category().name().to_string(),
        language_category_display: entry.language.category().label().to_string(),
        version: entry.version.to_string(),
        binary_name: entry.binary.to_string(),
        install_method: entry.install_method.label().to_string(),
        installed: installed_path.is_some(),
        install_state: state.to_string(),
        installed_path: installed_path.map(|p| p.display().to_string()),
        can_install: entry.install_method.is_managed()
            && entry.install_method.is_supported_on_current_platform(),
        can_remove: has_managed_install,
        footprint: entry.footprint.label().to_string(),
        maturity: entry.maturity.label().to_string(),
        source_url: entry.source_url.to_string(),
        rationale: entry.rationale.to_string(),
        manual_instructions: entry
            .install_method
            .manual_instructions()
            .map(ToOwned::to_owned),
        install_warning: entry
            .install_method
            .install_warning()
            .map(ToOwned::to_owned),
    }
}

fn find_on_path(entry: &ServerEntry) -> Option<PathBuf> {
    entry
        .aliases
        .iter()
        .chain(std::iter::once(&entry.binary))
        .find_map(|binary| which::which(binary).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_options_resolve_project_relative_cache_dir() {
        let root = tempfile::tempdir().unwrap();
        let config = ProjectConfig {
            cache_dir: Some(PathBuf::from(".cache/lsp-servers")),
            timeout: 12,
            prefer_path: false,
            overrides: Vec::new(),
        };

        let options = ServerOptions::from_config(root.path(), &config);

        assert_eq!(options.cache_dir, root.path().join(".cache/lsp-servers"));
        assert_eq!(options.timeout, Duration::from_secs(12));
        assert!(!options.prefer_path);
    }

    #[test]
    fn status_includes_install_warning_metadata() {
        let entry = REGISTRY.by_id("clangd").unwrap();
        let status = status_for_entry(entry);

        assert!(status.install_warning.is_some());
    }
}
