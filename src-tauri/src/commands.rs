use lsp_io_core::config::ProjectConfig;
use lsp_io_core::language::scan_languages;
use lsp_io_core::progress::{ProgressEvent, ProgressHandler};
use lsp_io_core::server::{
    InstallOutcome, ServerOptions, all_status_with_options, clean_managed_cache_with_options,
    install_server_with_options, remove_server_with_options,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
pub struct LanguageInfo {
    pub name: String,
    pub display_name: String,
    pub kind: String,
    pub category: String,
    pub category_display: String,
    pub confidence: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
}

struct TauriProgressHandler {
    app: AppHandle,
}

impl TauriProgressHandler {
    fn emit_frontend(&self, payload: serde_json::Value) {
        let _ = self.app.emit("progress", &payload);
    }
}

impl ProgressHandler for TauriProgressHandler {
    fn on_event(&self, event: ProgressEvent) {
        match event {
            ProgressEvent::ResolveStart { servers } => {
                self.emit_frontend(serde_json::json!({
                    "kind": "pipeline_step",
                    "step": "resolve",
                    "progress": 10,
                }));
                self.emit_frontend(serde_json::json!({
                    "kind": "log",
                    "level": "info",
                    "message": format!("Resolving install plan: {}", servers.join(", ")),
                }));
            }
            ProgressEvent::InstallStart { server, version } => {
                self.emit_frontend(serde_json::json!({
                    "kind": "server_progress",
                    "server": server,
                    "status": "installing",
                    "progress": 25,
                    "message": format!("Installing {version}"),
                }));
            }
            ProgressEvent::InstallOutput { server, line } => {
                self.emit_frontend(serde_json::json!({
                    "kind": "log",
                    "level": "info",
                    "message": format!("[{}] {}", server, line),
                }));
            }
            ProgressEvent::InstallComplete { server, path } => {
                self.emit_frontend(serde_json::json!({
                    "kind": "server_progress",
                    "server": server,
                    "status": "done",
                    "progress": 100,
                    "message": format!("Installed to {}", path.display()),
                }));
                self.emit_frontend(serde_json::json!({
                    "kind": "log",
                    "level": "success",
                    "message": format!("{} installed", server),
                }));
            }
            ProgressEvent::InstallFailed { server, error } => {
                self.emit_frontend(serde_json::json!({
                    "kind": "server_progress",
                    "server": server,
                    "status": "failed",
                    "progress": 0,
                    "message": error,
                }));
            }
            ProgressEvent::RemovalComplete { server, message } => {
                self.emit_frontend(serde_json::json!({
                    "kind": "log",
                    "level": "success",
                    "message": format!("{}: {}", server, message),
                }));
            }
        }
    }
}

#[tauri::command]
pub async fn detect_languages(path: String) -> Result<Vec<LanguageInfo>, String> {
    let root = PathBuf::from(&path);
    if !root.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let languages = scan_languages(&root).map_err(|e| e.to_string())?;
    Ok(languages
        .iter()
        .map(|language| LanguageInfo {
            name: language.name().to_string(),
            display_name: language.display_name().to_string(),
            kind: format!("{:?}", language.kind),
            category: language.category.name().to_string(),
            category_display: language.category.label().to_string(),
            confidence: language.confidence.label().to_string(),
            evidence: language.evidence.clone(),
        })
        .collect())
}

#[tauri::command]
pub async fn get_server_status(
    path: Option<String>,
) -> Result<Vec<lsp_io_core::server::ServerStatusInfo>, String> {
    let (root, config) = load_project_config(path.as_deref())?;
    let options = ServerOptions::from_config(&root, &config);
    Ok(all_status_with_options(&options))
}

#[tauri::command]
pub async fn install_servers(
    app: AppHandle,
    server_ids: Vec<String>,
    path: Option<String>,
) -> Result<Vec<InstallOutcome>, String> {
    let handler = TauriProgressHandler { app };
    let (root, config) = load_project_config(path.as_deref())?;
    let options = ServerOptions::from_config(&root, &config);
    handler.on_event(ProgressEvent::ResolveStart {
        servers: server_ids.clone(),
    });

    let mut outcomes = Vec::new();
    for id in server_ids {
        match install_server_with_options(&id, &options, &handler).await {
            Ok(outcome) => outcomes.push(outcome),
            Err(error) => outcomes.push(InstallOutcome {
                id: id.clone(),
                name: id,
                path: None,
                status: "failed".to_string(),
                message: error.to_string(),
            }),
        }
    }

    handler.emit_frontend(serde_json::json!({
        "kind": "pipeline_step",
        "step": "done",
        "progress": 100,
    }));
    handler.emit_frontend(serde_json::json!({
        "kind": "install_complete",
        "outcomes": outcomes,
    }));

    Ok(outcomes)
}

#[tauri::command]
pub async fn install_one_server(
    app: AppHandle,
    server_id: String,
    path: Option<String>,
) -> Result<InstallOutcome, String> {
    let handler = TauriProgressHandler { app };
    let (root, config) = load_project_config(path.as_deref())?;
    let options = ServerOptions::from_config(&root, &config);
    install_server_with_options(&server_id, &options, &handler)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_one_server(
    app: AppHandle,
    server_id: String,
    path: Option<String>,
) -> Result<InstallOutcome, String> {
    let handler = TauriProgressHandler { app };
    let (root, config) = load_project_config(path.as_deref())?;
    let options = ServerOptions::from_config(&root, &config);
    remove_server_with_options(&server_id, &options, &handler).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_config(path: String) -> Result<ProjectConfig, String> {
    let root = PathBuf::from(&path);
    ProjectConfig::load(&root).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_config(path: String, config: ProjectConfig) -> Result<(), String> {
    let config_path = PathBuf::from(&path).join(".lsp-io.toml");
    let toml_str = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, toml_str).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clean_cache(
    server_id: Option<String>,
    path: Option<String>,
) -> Result<String, String> {
    let (root, config) = load_project_config(path.as_deref())?;
    let options = ServerOptions::from_config(&root, &config);

    if let Some(id) = server_id {
        let handler = lsp_io_core::progress::NoopProgress;
        return remove_server_with_options(&id, &options, &handler)
            .map(|outcome| outcome.message)
            .map_err(|e| e.to_string());
    }

    let dir = options.cache_dir.clone();
    if dir.exists() {
        let removed = clean_managed_cache_with_options(&options).map_err(|e| e.to_string())?;
        Ok(format!(
            "Removed {removed} managed server install(s) from {}",
            dir.display()
        ))
    } else {
        Ok("Cache directory not found".to_string())
    }
}

#[tauri::command]
pub async fn check_updates() -> Result<Vec<UpdateInfo>, String> {
    Ok(lsp_io_core::server::REGISTRY
        .all()
        .iter()
        .map(|entry| UpdateInfo {
            id: entry.id.to_string(),
            name: entry.name.to_string(),
            language: entry.language.display_name().to_string(),
            current_version: entry.version.to_string(),
            latest_version: if entry.version == "system" {
                "system-managed".to_string()
            } else {
                "latest".to_string()
            },
            update_available: false,
        })
        .collect())
}

#[tauri::command]
pub async fn reveal_in_explorer(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    let dir = if p.is_file() {
        p.parent().unwrap_or(p)
    } else {
        p
    };

    #[cfg(target_os = "windows")]
    {
        if p.is_file() {
            std::process::Command::new("explorer")
                .args(["/select,", &path])
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            std::process::Command::new("explorer")
                .arg(dir)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn load_project_config(path: Option<&str>) -> Result<(PathBuf, ProjectConfig), String> {
    let root = match path {
        Some(path) if !path.trim().is_empty() => PathBuf::from(path),
        _ => std::env::current_dir().map_err(|e| e.to_string())?,
    };

    ProjectConfig::load(Path::new(&root))
        .map(|config| (root, config))
        .map_err(|e| e.to_string())
}
