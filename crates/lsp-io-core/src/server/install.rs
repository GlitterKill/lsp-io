use crate::progress::{ProgressEvent, ProgressHandler};
use crate::server::ServerOptions;
use crate::server::github_release::{self, GithubReleaseInstallInfo};
use crate::server::registry::{InstallMethod, REGISTRY, ServerEntry};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;

const INSTALL_MANIFEST: &str = "lsp-io-install.json";
const CACHE_MARKER: &str = ".lsp-io-cache";

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallOutcome {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstallManifest {
    id: String,
    install_id: String,
    name: String,
    version: String,
    binary: String,
    installed_path: String,
    source_kind: Option<String>,
    source_repo: Option<String>,
    release_tag: Option<String>,
    asset_name: Option<String>,
    asset_digest: Option<String>,
}

#[derive(Debug, Clone)]
struct InstallSourceMetadata {
    source_kind: String,
    source_repo: Option<String>,
    release_tag: Option<String>,
    asset_name: Option<String>,
    asset_digest: Option<String>,
}

#[derive(Debug, Clone)]
struct StagedInstall {
    binary: PathBuf,
    source: Option<InstallSourceMetadata>,
}

impl StagedInstall {
    fn plain(binary: PathBuf) -> Self {
        Self {
            binary,
            source: None,
        }
    }

    fn github(spec: &github_release::GithubReleaseSpec, info: GithubReleaseInstallInfo) -> Self {
        Self {
            binary: info.binary_path,
            source: Some(InstallSourceMetadata {
                source_kind: "github_release".to_string(),
                source_repo: Some(format!("{}/{}", spec.owner, spec.repo)),
                release_tag: Some(info.release_tag),
                asset_name: Some(info.asset_name),
                asset_digest: info.asset_digest,
            }),
        }
    }
}

pub async fn install_server(id: &str, progress: &dyn ProgressHandler) -> Result<InstallOutcome> {
    install_server_with_options(id, &ServerOptions::default(), progress).await
}

pub async fn install_server_with_options(
    id: &str,
    options: &ServerOptions,
    progress: &dyn ProgressHandler,
) -> Result<InstallOutcome> {
    let entry = REGISTRY
        .by_id(id)
        .ok_or_else(|| anyhow::anyhow!("Unknown language server: {id}"))?;

    if !entry.install_method.is_managed()
        || !entry.install_method.is_supported_on_current_platform()
    {
        bail!(
            "{} is not available for managed install. {}",
            entry.name,
            entry
                .install_method
                .manual_instructions()
                .unwrap_or("Install it with the upstream instructions.")
        );
    }

    if let Some(existing) = managed_install_path(entry, options) {
        return Ok(InstallOutcome {
            id: entry.id.to_string(),
            name: entry.name.to_string(),
            path: Some(existing.display().to_string()),
            status: "already_installed".to_string(),
            message: format!("{} is already managed by LSP-IO", entry.name),
        });
    }

    progress.on_event(ProgressEvent::InstallStart {
        server: entry.name.to_string(),
        version: entry.version.to_string(),
    });

    ensure_cache_root(options)?;
    let stage_root = staging_root(entry, options);
    std::fs::remove_dir_all(&stage_root).ok();
    std::fs::create_dir_all(&stage_root)?;

    let result = match &entry.install_method {
        InstallMethod::Npm { packages } => {
            install_npm(entry, packages, &stage_root, options, progress)
                .await
                .map(StagedInstall::plain)
        }
        InstallMethod::GoInstall { module } => {
            install_go(entry, module, &stage_root, options, progress)
                .await
                .map(StagedInstall::plain)
        }
        InstallMethod::DotnetTool { package } => {
            install_dotnet(entry, package, &stage_root, options, progress)
                .await
                .map(StagedInstall::plain)
        }
        InstallMethod::Gem { package } => {
            install_gem(entry, package, &stage_root, options, progress)
                .await
                .map(StagedInstall::plain)
        }
        InstallMethod::Pipx { package } => {
            install_pipx(entry, package, &stage_root, options, progress)
                .await
                .map(StagedInstall::plain)
        }
        InstallMethod::GithubReleaseArchive { spec } => {
            github_release::install_github_release_archive(
                entry,
                spec,
                &stage_root,
                options,
                progress,
            )
            .await
            .map(|info| StagedInstall::github(spec, info))
        }
        InstallMethod::Manual { .. } => unreachable!("manual install handled above"),
    };

    match result {
        Ok(staged) => {
            let final_binary =
                final_path_for_stage_binary(entry, options, &stage_root, &staged.binary);
            write_manifest(&stage_root, entry, &final_binary, staged.source.as_ref())?;
            promote_stage(entry, options, &stage_root)?;

            if !final_binary.exists() {
                bail!(
                    "install finished but {} was not found under {}",
                    entry.binary,
                    server_root(entry, options).display()
                );
            }

            progress.on_event(ProgressEvent::InstallComplete {
                server: entry.name.to_string(),
                path: final_binary.clone(),
            });
            Ok(InstallOutcome {
                id: entry.id.to_string(),
                name: entry.name.to_string(),
                path: Some(final_binary.display().to_string()),
                status: "installed".to_string(),
                message: format!("Installed {}", entry.name),
            })
        }
        Err(error) => {
            std::fs::remove_dir_all(&stage_root).ok();
            progress.on_event(ProgressEvent::InstallFailed {
                server: entry.name.to_string(),
                error: error.to_string(),
            });
            Err(error)
        }
    }
}

pub fn remove_server(id: &str, progress: &dyn ProgressHandler) -> Result<InstallOutcome> {
    remove_server_with_options(id, &ServerOptions::default(), progress)
}

pub fn remove_server_with_options(
    id: &str,
    options: &ServerOptions,
    progress: &dyn ProgressHandler,
) -> Result<InstallOutcome> {
    let entry = REGISTRY
        .by_id(id)
        .ok_or_else(|| anyhow::anyhow!("Unknown language server: {id}"))?;

    if !entry.install_method.is_managed() {
        bail!(
            "{} is system/manual; system installs are never removed",
            entry.name
        );
    }

    let root = server_root(entry, options);
    if !root.exists() {
        bail!(
            "{} is not installed in the LSP-IO managed cache; system installs are never removed",
            entry.name
        );
    }
    if !manifest_owned_by_entry(entry, options) {
        bail!(
            "{} does not have an LSP-IO install manifest; refusing to remove {}",
            entry.name,
            root.display()
        );
    }

    std::fs::remove_dir_all(&root)
        .with_context(|| format!("Failed to remove {}", root.display()))?;

    let message = format!("Removed managed {} install", entry.name);
    progress.on_event(ProgressEvent::RemovalComplete {
        server: entry.name.to_string(),
        message: message.clone(),
    });

    Ok(InstallOutcome {
        id: entry.id.to_string(),
        name: entry.name.to_string(),
        path: None,
        status: "removed".to_string(),
        message,
    })
}

pub fn managed_binary_path(entry: &ServerEntry) -> PathBuf {
    managed_binary_path_with_options(entry, &ServerOptions::default())
}

pub fn managed_binary_path_with_options(entry: &ServerEntry, options: &ServerOptions) -> PathBuf {
    let root = server_root(entry, options);
    binary_path_under(entry, &root)
}

pub fn managed_install_path(entry: &ServerEntry, options: &ServerOptions) -> Option<PathBuf> {
    let binary = managed_binary_path_with_options(entry, options);
    if binary.exists() && manifest_matches(entry, options) {
        Some(binary)
    } else {
        None
    }
}

pub fn clean_managed_cache_with_options(options: &ServerOptions) -> Result<usize> {
    let mut removed = 0;

    for entry in REGISTRY.all() {
        let root = server_root(entry, options);
        if root.exists() && manifest_owned_by_entry(entry, options) {
            std::fs::remove_dir_all(&root)
                .with_context(|| format!("Failed to remove {}", root.display()))?;
            removed += 1;
        }
    }

    let staging = options.cache_dir.join(".staging");
    if staging.exists() && cache_marker_exists(options) {
        std::fs::remove_dir_all(&staging)
            .with_context(|| format!("Failed to remove {}", staging.display()))?;
    }

    Ok(removed)
}

fn server_root(entry: &ServerEntry, options: &ServerOptions) -> PathBuf {
    options.cache_dir.join(entry.install_id())
}

fn staging_root(entry: &ServerEntry, options: &ServerOptions) -> PathBuf {
    options.cache_dir.join(".staging").join(format!(
        "{}-{}",
        entry.install_id(),
        std::process::id()
    ))
}

fn ensure_cache_root(options: &ServerOptions) -> Result<()> {
    std::fs::create_dir_all(&options.cache_dir)
        .with_context(|| format!("Failed to create {}", options.cache_dir.display()))?;
    std::fs::write(
        options.cache_dir.join(CACHE_MARKER),
        "LSP-IO managed cache root\n",
    )
    .with_context(|| format!("Failed to mark {}", options.cache_dir.display()))?;
    Ok(())
}

fn cache_marker_exists(options: &ServerOptions) -> bool {
    options.cache_dir.join(CACHE_MARKER).is_file()
}

fn binary_path_under(entry: &ServerEntry, root: &Path) -> PathBuf {
    match entry.install_method {
        InstallMethod::Npm { .. } => root
            .join("npm")
            .join("node_modules")
            .join(".bin")
            .join(binary_with_cmd(entry.binary)),
        InstallMethod::GoInstall { .. } => root
            .join("go")
            .join("bin")
            .join(binary_with_exe(entry.binary)),
        InstallMethod::DotnetTool { .. } => root
            .join("dotnet-tools")
            .join(binary_with_exe(entry.binary)),
        InstallMethod::Gem { .. } => root
            .join("gems")
            .join("bin")
            .join(binary_with_bat(entry.binary)),
        InstallMethod::Pipx { .. } => root
            .join("pipx")
            .join("bin")
            .join(binary_with_exe(entry.binary)),
        InstallMethod::GithubReleaseArchive { spec } => {
            github_release::binary_path_in_root(spec, &root.join("github-release")).unwrap_or_else(
                || {
                    root.join("github-release")
                        .join(binary_with_exe(entry.binary))
                },
            )
        }
        InstallMethod::Manual { .. } => root.join("manual").join(binary_with_exe(entry.binary)),
    }
}

async fn install_npm(
    entry: &ServerEntry,
    packages: &[&str],
    root: &Path,
    options: &ServerOptions,
    progress: &dyn ProgressHandler,
) -> Result<PathBuf> {
    let npm = which::which("npm").context("npm not found on PATH. Install Node.js first.")?;
    let prefix = root.join("npm");
    std::fs::create_dir_all(&prefix)?;

    let package_specs: Vec<String> = packages
        .iter()
        .map(|package| format!("{package}@latest"))
        .collect();
    run_command(
        &npm,
        ["install", "--prefix"]
            .into_iter()
            .map(String::from)
            .chain(std::iter::once(prefix.display().to_string()).chain(package_specs)),
        entry,
        progress,
        options,
    )
    .await?;

    expect_binary_at(entry, root)
}

async fn install_go(
    entry: &ServerEntry,
    module: &str,
    root: &Path,
    options: &ServerOptions,
    progress: &dyn ProgressHandler,
) -> Result<PathBuf> {
    let go = which::which("go").context("go not found on PATH. Install Go first.")?;
    let gobin = root.join("go").join("bin");
    std::fs::create_dir_all(&gobin)?;

    let spec = if entry.version == "latest" {
        format!("{module}@latest")
    } else {
        format!("{module}@{}", entry.version)
    };

    let mut command = Command::new(&go);
    command.env("GOBIN", &gobin).arg("install").arg(spec);
    run_prepared_command(command, entry, progress, options).await?;

    expect_binary_at(entry, root)
}

async fn install_dotnet(
    entry: &ServerEntry,
    package: &str,
    root: &Path,
    options: &ServerOptions,
    progress: &dyn ProgressHandler,
) -> Result<PathBuf> {
    let dotnet =
        which::which("dotnet").context("dotnet not found on PATH. Install the .NET SDK first.")?;
    let tool_dir = root.join("dotnet-tools");
    std::fs::create_dir_all(&tool_dir)?;

    let mut args = vec![
        "tool".to_string(),
        "install".to_string(),
        "--tool-path".to_string(),
        tool_dir.display().to_string(),
        package.to_string(),
    ];
    if entry.version != "latest" {
        args.push("--version".to_string());
        args.push(entry.version.to_string());
    }

    run_command(&dotnet, args, entry, progress, options).await?;
    expect_binary_at(entry, root)
}

async fn install_gem(
    entry: &ServerEntry,
    package: &str,
    root: &Path,
    options: &ServerOptions,
    progress: &dyn ProgressHandler,
) -> Result<PathBuf> {
    let gem = which::which("gem").context("gem not found on PATH. Install Ruby first.")?;
    let gem_home = root.join("gems");
    let bindir = gem_home.join("bin");
    std::fs::create_dir_all(&bindir)?;

    let mut args = vec![
        "install".to_string(),
        "--install-dir".to_string(),
        gem_home.display().to_string(),
        "--bindir".to_string(),
        bindir.display().to_string(),
        package.to_string(),
    ];
    if entry.version != "latest" {
        args.push("--version".to_string());
        args.push(entry.version.to_string());
    }

    run_command(&gem, args, entry, progress, options).await?;
    expect_binary_at(entry, root)
}

async fn install_pipx(
    entry: &ServerEntry,
    package: &str,
    root: &Path,
    options: &ServerOptions,
    progress: &dyn ProgressHandler,
) -> Result<PathBuf> {
    let pipx = which::which("pipx").context("pipx not found on PATH. Install pipx first.")?;
    let pipx_home = root.join("pipx").join("home");
    let pipx_bin = root.join("pipx").join("bin");
    std::fs::create_dir_all(&pipx_home)?;
    std::fs::create_dir_all(&pipx_bin)?;

    let spec = if entry.version == "latest" {
        package.to_string()
    } else {
        format!("{package}=={}", entry.version)
    };

    let mut command = Command::new(&pipx);
    command
        .env("PIPX_HOME", &pipx_home)
        .env("PIPX_BIN_DIR", &pipx_bin)
        .arg("install")
        .arg(spec);
    run_prepared_command(command, entry, progress, options).await?;

    expect_binary_at(entry, root)
}

async fn run_command<I, S>(
    program: &Path,
    args: I,
    entry: &ServerEntry,
    progress: &dyn ProgressHandler,
    options: &ServerOptions,
) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut command = Command::new(program);
    command.args(args);
    run_prepared_command(command, entry, progress, options).await
}

async fn run_prepared_command(
    mut command: Command,
    entry: &ServerEntry,
    progress: &dyn ProgressHandler,
    options: &ServerOptions,
) -> Result<()> {
    let output = tokio::time::timeout(options.timeout, command.output())
        .await
        .with_context(|| {
            format!(
                "installer for {} timed out after {} seconds",
                entry.name,
                options.timeout.as_secs()
            )
        })?
        .with_context(|| format!("Failed to run installer for {}", entry.name))?;

    emit_output(entry, progress, &output.stdout);
    emit_output(entry, progress, &output.stderr);

    if !output.status.success() {
        bail!("installer for {} exited with {}", entry.name, output.status);
    }

    Ok(())
}

fn emit_output(entry: &ServerEntry, progress: &dyn ProgressHandler, bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes);
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        progress.on_event(ProgressEvent::InstallOutput {
            server: entry.name.to_string(),
            line: line.to_string(),
        });
    }
}

fn expect_binary_at(entry: &ServerEntry, root: &Path) -> Result<PathBuf> {
    let primary = binary_path_under(entry, root);
    if primary.exists() {
        return Ok(primary);
    }

    // Ruby and npm wrappers vary by platform, so check aliases before failing.
    let candidates = [
        primary.with_extension("cmd"),
        primary.with_extension("bat"),
        primary.with_extension("exe"),
        primary.with_extension(""),
    ];
    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    bail!(
        "install finished but {} was not found under {}",
        entry.binary,
        root.display()
    )
}

fn final_path_for_stage_binary(
    entry: &ServerEntry,
    options: &ServerOptions,
    stage_root: &Path,
    stage_binary: &Path,
) -> PathBuf {
    let final_root = server_root(entry, options);
    if let Ok(relative) = stage_binary.strip_prefix(stage_root) {
        final_root.join(relative)
    } else {
        managed_binary_path_with_options(entry, options)
    }
}

fn write_manifest(
    root: &Path,
    entry: &ServerEntry,
    installed_path: &Path,
    source: Option<&InstallSourceMetadata>,
) -> Result<()> {
    let manifest = InstallManifest {
        id: entry.id.to_string(),
        install_id: entry.install_id().to_string(),
        name: entry.name.to_string(),
        version: entry.version.to_string(),
        binary: entry.binary.to_string(),
        installed_path: installed_path.display().to_string(),
        source_kind: source.map(|source| source.source_kind.clone()),
        source_repo: source.and_then(|source| source.source_repo.clone()),
        release_tag: source.and_then(|source| source.release_tag.clone()),
        asset_name: source.and_then(|source| source.asset_name.clone()),
        asset_digest: source.and_then(|source| source.asset_digest.clone()),
    };
    let raw = serde_json::to_string_pretty(&manifest)?;
    std::fs::write(root.join(INSTALL_MANIFEST), raw)?;
    Ok(())
}

fn promote_stage(entry: &ServerEntry, options: &ServerOptions, stage_root: &Path) -> Result<()> {
    let final_root = server_root(entry, options);
    if let Some(parent) = final_root.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if final_root.exists() {
        if !manifest_owned_by_entry(entry, options) {
            bail!(
                "refusing to replace {}; no LSP-IO install manifest was found",
                final_root.display()
            );
        }
        std::fs::remove_dir_all(&final_root)
            .with_context(|| format!("Failed to replace {}", final_root.display()))?;
    }
    std::fs::rename(stage_root, &final_root).with_context(|| {
        format!(
            "Failed to promote staged install {} to {}",
            stage_root.display(),
            final_root.display()
        )
    })?;
    Ok(())
}

fn manifest_matches(entry: &ServerEntry, options: &ServerOptions) -> bool {
    let Some(manifest) = read_manifest(entry, options) else {
        return false;
    };

    if manifest.install_id != entry.install_id() || manifest.version != entry.version {
        return false;
    }

    manifest_source_matches(entry, &manifest)
}

fn manifest_owned_by_entry(entry: &ServerEntry, options: &ServerOptions) -> bool {
    read_manifest(entry, options)
        .map(|manifest| manifest.install_id == entry.install_id())
        .unwrap_or(false)
}

fn read_manifest(entry: &ServerEntry, options: &ServerOptions) -> Option<InstallManifest> {
    let path = server_root(entry, options).join(INSTALL_MANIFEST);
    let Ok(raw) = std::fs::read_to_string(path) else {
        return None;
    };
    let Ok(manifest) = serde_json::from_str::<InstallManifest>(&raw) else {
        return None;
    };

    Some(manifest)
}

fn manifest_source_matches(entry: &ServerEntry, manifest: &InstallManifest) -> bool {
    match &entry.install_method {
        InstallMethod::GithubReleaseArchive { spec } => {
            let Ok(asset) = github_release::asset_for_current_platform(spec) else {
                return false;
            };
            let expected_repo = format!("{}/{}", spec.owner, spec.repo);

            manifest.source_kind.as_deref() == Some("github_release")
                && manifest.source_repo.as_deref() == Some(expected_repo.as_str())
                && manifest.asset_name.as_deref() == Some(asset.asset_name)
                && match spec.selector {
                    github_release::ReleaseSelector::Tag(tag) => {
                        manifest.release_tag.as_deref() == Some(tag)
                    }
                    github_release::ReleaseSelector::Latest => manifest.release_tag.is_some(),
                }
        }
        _ => manifest.source_kind.is_none(),
    }
}

fn binary_with_exe(binary: &str) -> String {
    if cfg!(windows) && !binary.ends_with(".exe") {
        format!("{binary}.exe")
    } else {
        binary.to_string()
    }
}

fn binary_with_cmd(binary: &str) -> String {
    if cfg!(windows) && !binary.ends_with(".cmd") {
        format!("{binary}.cmd")
    } else {
        binary.to_string()
    }
}

fn binary_with_bat(binary: &str) -> String {
    if cfg!(windows) && !binary.ends_with(".bat") {
        format!("{binary}.bat")
    } else {
        binary.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::NoopProgress;

    #[test]
    fn manual_remove_is_non_destructive() {
        let result = remove_server("jdtls", &NoopProgress);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("system"));
    }

    #[test]
    fn managed_paths_are_server_scoped() {
        let ts = REGISTRY.by_id("typescript-language-server").unwrap();
        let py = REGISTRY.by_id("pyright").unwrap();
        assert!(managed_binary_path(ts).starts_with(server_root(ts, &ServerOptions::default())));
        assert!(managed_binary_path(py).starts_with(server_root(py, &ServerOptions::default())));
        assert_ne!(
            server_root(ts, &ServerOptions::default()),
            server_root(py, &ServerOptions::default())
        );
    }

    #[test]
    fn javascript_and_typescript_share_install_root() {
        let ts = REGISTRY.by_id("typescript-language-server").unwrap();
        let js = REGISTRY.by_id("javascript-language-server").unwrap();

        assert_eq!(
            server_root(ts, &ServerOptions::default()),
            server_root(js, &ServerOptions::default())
        );
    }

    #[test]
    fn shared_package_servers_accept_one_success_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let options = ServerOptions {
            cache_dir: dir.path().to_path_buf(),
            ..ServerOptions::default()
        };
        let html = REGISTRY.by_id("html-language-server").unwrap();
        let css = REGISTRY.by_id("css-language-server").unwrap();

        let html_binary = managed_binary_path_with_options(html, &options);
        let css_binary = managed_binary_path_with_options(css, &options);
        std::fs::create_dir_all(html_binary.parent().unwrap()).unwrap();
        std::fs::write(&html_binary, "").unwrap();
        std::fs::write(&css_binary, "").unwrap();
        write_manifest(&server_root(html, &options), html, &html_binary, None).unwrap();

        assert!(managed_install_path(html, &options).is_some());
        assert!(managed_install_path(css, &options).is_some());
    }

    #[test]
    fn github_release_manifest_requires_current_source_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let options = ServerOptions {
            cache_dir: dir.path().to_path_buf(),
            ..ServerOptions::default()
        };
        let entry = REGISTRY.by_id("rust-analyzer").unwrap();
        let binary = managed_binary_path_with_options(entry, &options);
        let source = github_source_for_current_platform(entry);

        std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
        std::fs::write(&binary, "").unwrap();
        write_manifest(&server_root(entry, &options), entry, &binary, Some(&source)).unwrap();
        assert!(managed_install_path(entry, &options).is_some());

        let stale_source = InstallSourceMetadata {
            asset_name: Some("old-rust-analyzer.gz".to_string()),
            ..source
        };
        write_manifest(
            &server_root(entry, &options),
            entry,
            &binary,
            Some(&stale_source),
        )
        .unwrap();

        assert!(managed_install_path(entry, &options).is_none());
        assert!(manifest_owned_by_entry(entry, &options));
    }

    #[test]
    fn stale_owned_manifest_can_be_replaced_by_promotion() {
        let dir = tempfile::tempdir().unwrap();
        let options = ServerOptions {
            cache_dir: dir.path().to_path_buf(),
            ..ServerOptions::default()
        };
        let entry = REGISTRY.by_id("pyright").unwrap();
        let root = server_root(entry, &options);
        let stage = staging_root(entry, &options);
        let binary = managed_binary_path_with_options(entry, &options);
        let stale_manifest = InstallManifest {
            id: entry.id.to_string(),
            install_id: entry.install_id().to_string(),
            name: entry.name.to_string(),
            version: "old".to_string(),
            binary: entry.binary.to_string(),
            installed_path: binary.display().to_string(),
            source_kind: None,
            source_repo: None,
            release_tag: None,
            asset_name: None,
            asset_digest: None,
        };

        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join(INSTALL_MANIFEST),
            serde_json::to_string(&stale_manifest).unwrap(),
        )
        .unwrap();
        std::fs::write(root.join("old-file"), "").unwrap();
        std::fs::create_dir_all(&stage).unwrap();
        std::fs::write(stage.join("new-file"), "").unwrap();

        assert!(managed_install_path(entry, &options).is_none());
        assert!(manifest_owned_by_entry(entry, &options));

        promote_stage(entry, &options, &stage).unwrap();

        assert!(root.join("new-file").exists());
        assert!(!root.join("old-file").exists());
    }

    #[test]
    fn remove_refuses_directory_without_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let options = ServerOptions {
            cache_dir: dir.path().to_path_buf(),
            ..ServerOptions::default()
        };
        let entry = REGISTRY.by_id("pyright").unwrap();
        let root = server_root(entry, &options);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("user-file.txt"), "keep").unwrap();

        let result = remove_server_with_options(entry.id, &options, &NoopProgress);

        assert!(result.is_err());
        assert!(root.join("user-file.txt").exists());
    }

    #[test]
    fn clean_cache_removes_only_manifest_owned_server_roots() {
        let dir = tempfile::tempdir().unwrap();
        let options = ServerOptions {
            cache_dir: dir.path().to_path_buf(),
            ..ServerOptions::default()
        };
        let owned = REGISTRY.by_id("pyright").unwrap();
        let unowned = REGISTRY.by_id("gopls").unwrap();
        let owned_binary = managed_binary_path_with_options(owned, &options);
        std::fs::create_dir_all(owned_binary.parent().unwrap()).unwrap();
        std::fs::write(&owned_binary, "").unwrap();
        write_manifest(&server_root(owned, &options), owned, &owned_binary, None).unwrap();

        let unowned_root = server_root(unowned, &options);
        std::fs::create_dir_all(&unowned_root).unwrap();
        std::fs::write(unowned_root.join("user-file.txt"), "keep").unwrap();

        let removed = clean_managed_cache_with_options(&options).unwrap();

        assert_eq!(removed, 1);
        assert!(!server_root(owned, &options).exists());
        assert!(unowned_root.join("user-file.txt").exists());
        assert!(dir.path().exists());
    }

    #[test]
    fn binary_without_manifest_is_not_managed_install() {
        let dir = tempfile::tempdir().unwrap();
        let options = ServerOptions {
            cache_dir: dir.path().to_path_buf(),
            ..ServerOptions::default()
        };
        let entry = REGISTRY.by_id("pyright").unwrap();
        let binary = managed_binary_path_with_options(entry, &options);
        std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
        std::fs::write(&binary, "").unwrap();

        assert!(managed_install_path(entry, &options).is_none());
    }

    #[test]
    fn github_release_archive_path_uses_expected_binary_mapping() {
        let dir = tempfile::tempdir().unwrap();
        let options = ServerOptions {
            cache_dir: dir.path().to_path_buf(),
            ..ServerOptions::default()
        };
        let entry = REGISTRY.by_id("ada-language-server").unwrap();

        assert_eq!(entry.install_method.label(), "github release");
        assert!(entry.install_method.is_managed());
        assert!(
            managed_binary_path_with_options(entry, &options)
                .starts_with(server_root(entry, &options).join("github-release"))
        );
    }

    fn github_source_for_current_platform(entry: &ServerEntry) -> InstallSourceMetadata {
        let InstallMethod::GithubReleaseArchive { spec } = &entry.install_method else {
            panic!("expected GitHub release install method");
        };
        let asset = github_release::asset_for_current_platform(spec).unwrap();
        let release_tag = match spec.selector {
            github_release::ReleaseSelector::Tag(tag) => tag.to_string(),
            github_release::ReleaseSelector::Latest => "latest".to_string(),
        };

        InstallSourceMetadata {
            source_kind: "github_release".to_string(),
            source_repo: Some(format!("{}/{}", spec.owner, spec.repo)),
            release_tag: Some(release_tag),
            asset_name: Some(asset.asset_name.to_string()),
            asset_digest: None,
        }
    }
}
