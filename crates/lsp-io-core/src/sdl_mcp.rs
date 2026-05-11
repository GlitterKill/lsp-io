use crate::config::ProjectConfig;
use crate::language::scan_languages;
use crate::server::{
    REGISTRY, SdlMcpReadiness, ServerEntry, ServerOptions, status_for_entry_with_options,
};
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Default)]
pub struct SdlMcpExportOptions {
    pub include_missing: bool,
    pub validate_launch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SdlMcpLspServerConfig {
    pub enabled: bool,
    pub server_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub languages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub document_language_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_patterns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialization_options: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    pub readiness: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SdlMcpExportFragment {
    #[serde(rename = "semanticEnrichment")]
    pub semantic_enrichment: SdlMcpSemanticEnrichmentFragment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SdlMcpSemanticEnrichmentFragment {
    pub providers: SdlMcpProvidersFragment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SdlMcpProvidersFragment {
    pub lsp: SdlMcpLspProviderFragment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SdlMcpLspProviderFragment {
    pub servers: BTreeMap<String, SdlMcpLspServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SdlMcpExportDiagnostic {
    pub server_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SdlMcpExportResult {
    pub fragment: SdlMcpExportFragment,
    pub diagnostics: Vec<SdlMcpExportDiagnostic>,
}

impl SdlMcpExportResult {
    pub fn server_count(&self) -> usize {
        self.fragment
            .semantic_enrichment
            .providers
            .lsp
            .servers
            .len()
    }
}

pub fn build_sdl_mcp_export(
    project_root: &Path,
    config: &ProjectConfig,
    options: SdlMcpExportOptions,
) -> Result<SdlMcpExportResult> {
    let detected_languages = scan_languages(project_root)?
        .into_iter()
        .map(|language| language.kind)
        .collect::<HashSet<_>>();

    let mut entries = BTreeMap::<&str, &ServerEntry>::new();
    for language in detected_languages {
        for entry in REGISTRY.for_language(language) {
            entries.insert(entry.id, entry);
        }
    }

    // Explicit overrides are operator intent, so include them even if the
    // current repository scan did not detect that language.
    for override_entry in &config.overrides {
        if let Some(entry) = REGISTRY.by_id(&override_entry.id) {
            entries.insert(entry.id, entry);
        }
    }

    let server_options = ServerOptions::from_config(project_root, config);
    let mut servers = BTreeMap::new();
    let mut diagnostics = Vec::new();

    for entry in entries.values() {
        let metadata = entry.sdl_mcp_metadata();
        let override_entry = config.override_for(entry.id);
        let command = resolve_command(
            project_root,
            entry,
            &server_options,
            override_entry.and_then(|override_entry| override_entry.binary_path.as_ref()),
            options.include_missing,
        );

        let Some(command) = command else {
            diagnostics.push(SdlMcpExportDiagnostic {
                server_id: entry.id.to_string(),
                reason: "missing command; install the server or configure an override".to_string(),
            });
            continue;
        };

        if options.validate_launch && !command_is_resolved(&command) {
            diagnostics.push(SdlMcpExportDiagnostic {
                server_id: entry.id.to_string(),
                reason: format!("launch command is not currently resolvable: {command}"),
            });
        }

        let args = override_entry
            .filter(|override_entry| !override_entry.args.is_empty())
            .map(|override_entry| override_entry.args.clone())
            .unwrap_or_else(|| metadata.args.clone());

        let initialization_options = metadata
            .initialization_options_json
            .map(serde_json::from_str)
            .transpose()
            .with_context(|| format!("invalid SDL-MCP initialization options for {}", entry.id))?;

        let readiness = if !command_is_resolved(&command) {
            "missing"
        } else {
            match metadata.readiness {
                SdlMcpReadiness::Managed => "ready",
                SdlMcpReadiness::Manual => "manual",
                SdlMcpReadiness::UnsupportedPlatform => "unsupported_platform",
            }
        };

        servers.insert(
            entry.id.to_string(),
            SdlMcpLspServerConfig {
                enabled: true,
                server_id: metadata.server_id,
                command,
                args,
                languages: metadata.languages,
                document_language_ids: metadata.document_language_ids,
                file_patterns: metadata.file_patterns,
                initialization_options,
                capabilities: metadata.capabilities,
                readiness: readiness.to_string(),
            },
        );
    }

    Ok(SdlMcpExportResult {
        fragment: SdlMcpExportFragment {
            semantic_enrichment: SdlMcpSemanticEnrichmentFragment {
                providers: SdlMcpProvidersFragment {
                    lsp: SdlMcpLspProviderFragment { servers },
                },
            },
        },
        diagnostics,
    })
}

pub fn merge_sdl_mcp_fragment(
    existing: Value,
    fragment: &SdlMcpExportFragment,
    enable_semantic_enrichment: bool,
) -> Result<Value> {
    let mut root = match existing {
        Value::Object(object) => object,
        Value::Null => Map::new(),
        _ => return Err(anyhow!("SDL-MCP config root must be a JSON object")),
    };

    let semantic = ensure_object(&mut root, "semanticEnrichment")?;
    if enable_semantic_enrichment {
        semantic.insert("enabled".to_string(), Value::Bool(true));
    }

    let providers = ensure_object(semantic, "providers")?;
    let lsp = ensure_object(providers, "lsp")?;
    lsp.entry("enabled".to_string())
        .or_insert(Value::Bool(true));
    let servers = ensure_object(lsp, "servers")?;

    let fragment_value = serde_json::to_value(fragment)?;
    let fragment_servers = fragment_value
        .pointer("/semanticEnrichment/providers/lsp/servers")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("internal SDL-MCP fragment is missing LSP servers"))?;

    for (key, value) in fragment_servers {
        servers.insert(key.clone(), value.clone());
    }

    Ok(Value::Object(root))
}

pub fn write_sdl_mcp_config(
    project_root: &Path,
    config_path: &Path,
    config: &ProjectConfig,
    options: SdlMcpExportOptions,
    enable_semantic_enrichment: bool,
) -> Result<SdlMcpExportResult> {
    let export = build_sdl_mcp_export(project_root, config, options)?;
    let existing = if config_path.exists() {
        let raw = std::fs::read_to_string(config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;
        serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse {}", config_path.display()))?
    } else {
        json!({})
    };

    let merged = merge_sdl_mcp_fragment(existing, &export.fragment, enable_semantic_enrichment)?;
    let formatted = serde_json::to_string_pretty(&merged)?;
    std::fs::write(config_path, format!("{formatted}\n"))
        .with_context(|| format!("failed to write {}", config_path.display()))?;

    Ok(export)
}

fn resolve_command(
    project_root: &Path,
    entry: &ServerEntry,
    options: &ServerOptions,
    override_path: Option<&PathBuf>,
    include_missing: bool,
) -> Option<String> {
    if let Some(path) = override_path {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            project_root.join(path)
        };
        return Some(resolved.display().to_string());
    }

    let status = status_for_entry_with_options(entry, options);
    if let Some(installed_path) = status.installed_path {
        return Some(installed_path);
    }

    include_missing.then(|| entry.binary.to_string())
}

fn command_is_resolved(command: &str) -> bool {
    let path = Path::new(command);
    if path.is_absolute() || command.contains('\\') || command.contains('/') {
        return path.exists();
    }

    which::which(command).is_ok()
}

fn ensure_object<'a>(
    parent: &'a mut Map<String, Value>,
    key: &str,
) -> Result<&'a mut Map<String, Value>> {
    if !parent.contains_key(key) {
        parent.insert(key.to_string(), Value::Object(Map::new()));
    }

    parent
        .get_mut(key)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| anyhow!("{key} must be a JSON object"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn export_uses_override_command_and_registry_args() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("tsconfig.json"), "{}").unwrap();
        let config = ProjectConfig {
            overrides: vec![crate::config::ServerOverride {
                id: "typescript-language-server".to_string(),
                binary_path: Some(PathBuf::from("tools/typescript-language-server.cmd")),
                args: Vec::new(),
            }],
            ..ProjectConfig::default()
        };

        let export =
            build_sdl_mcp_export(root.path(), &config, SdlMcpExportOptions::default()).unwrap();
        let server = export
            .fragment
            .semantic_enrichment
            .providers
            .lsp
            .servers
            .get("typescript-language-server")
            .unwrap();

        assert_eq!(
            server.command,
            root.path()
                .join("tools/typescript-language-server.cmd")
                .display()
                .to_string()
        );
        assert_eq!(server.args, ["--stdio"]);
        assert_eq!(server.languages, ["typescript"]);
    }

    #[test]
    fn export_override_args_replace_registry_args() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("tsconfig.json"), "{}").unwrap();
        let config = ProjectConfig {
            overrides: vec![crate::config::ServerOverride {
                id: "typescript-language-server".to_string(),
                binary_path: Some(PathBuf::from("tls")),
                args: vec![
                    "--stdio".to_string(),
                    "--log-level".to_string(),
                    "4".to_string(),
                ],
            }],
            ..ProjectConfig::default()
        };

        let export =
            build_sdl_mcp_export(root.path(), &config, SdlMcpExportOptions::default()).unwrap();
        let server = export
            .fragment
            .semantic_enrichment
            .providers
            .lsp
            .servers
            .get("typescript-language-server")
            .unwrap();

        assert_eq!(server.args, ["--stdio", "--log-level", "4"]);
    }

    #[test]
    fn include_missing_emits_preview_command() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("Puppetfile"), "").unwrap();
        let export = build_sdl_mcp_export(
            root.path(),
            &ProjectConfig::default(),
            SdlMcpExportOptions {
                include_missing: true,
                validate_launch: false,
            },
        )
        .unwrap();

        let server = export
            .fragment
            .semantic_enrichment
            .providers
            .lsp
            .servers
            .get("puppet-editor-services")
            .unwrap();

        assert_eq!(server.command, "puppet-languageserver");
        assert_eq!(server.readiness, "missing");
    }

    #[test]
    fn merge_preserves_unrelated_config_and_unknown_servers() {
        let mut servers = BTreeMap::new();
        servers.insert(
            "typescript-language-server".to_string(),
            SdlMcpLspServerConfig {
                enabled: true,
                server_id: "typescript-language-server".to_string(),
                command: "typescript-language-server".to_string(),
                args: vec!["--stdio".to_string()],
                languages: vec!["typescript".to_string()],
                document_language_ids: vec!["typescript".to_string()],
                file_patterns: vec!["**/*.ts".to_string()],
                initialization_options: None,
                capabilities: vec!["documentSymbol".to_string()],
                readiness: "missing".to_string(),
            },
        );
        let fragment = SdlMcpExportFragment {
            semantic_enrichment: SdlMcpSemanticEnrichmentFragment {
                providers: SdlMcpProvidersFragment {
                    lsp: SdlMcpLspProviderFragment { servers },
                },
            },
        };
        let existing = json!({
            "repos": [{"path": "repo"}],
            "semanticEnrichment": {
                "providers": {
                    "lsp": {
                        "servers": {
                            "manual": {"serverId": "manual", "command": "manual", "languages": []}
                        }
                    }
                }
            }
        });

        let merged = merge_sdl_mcp_fragment(existing, &fragment, true).unwrap();

        assert_eq!(merged.pointer("/repos/0/path").unwrap(), "repo");
        assert_eq!(merged.pointer("/semanticEnrichment/enabled").unwrap(), true);
        assert!(
            merged
                .pointer("/semanticEnrichment/providers/lsp/servers/manual")
                .is_some()
        );
        assert!(
            merged
                .pointer("/semanticEnrichment/providers/lsp/servers/typescript-language-server")
                .is_some()
        );
    }

    #[test]
    fn registry_metadata_covers_all_entries_for_export() {
        for entry in REGISTRY.all() {
            let metadata = entry.sdl_mcp_metadata();

            assert_eq!(metadata.server_id, entry.id);
            assert!(!metadata.languages.is_empty(), "{}", entry.id);
            assert!(!metadata.document_language_ids.is_empty(), "{}", entry.id);
            assert!(!metadata.capabilities.is_empty(), "{}", entry.id);
        }
    }

    #[test]
    fn project_export_includes_overrides_even_without_detection() {
        let root = tempfile::tempdir().unwrap();
        let config = ProjectConfig {
            overrides: vec![crate::config::ServerOverride {
                id: "rust-analyzer".to_string(),
                binary_path: Some(PathBuf::from("rust-analyzer")),
                args: Vec::new(),
            }],
            ..ProjectConfig::default()
        };

        let export =
            build_sdl_mcp_export(root.path(), &config, SdlMcpExportOptions::default()).unwrap();

        assert!(
            export
                .fragment
                .semantic_enrichment
                .providers
                .lsp
                .servers
                .contains_key("rust-analyzer")
        );
    }
}
