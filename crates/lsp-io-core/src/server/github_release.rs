use crate::progress::{ProgressEvent, ProgressHandler};
use crate::server::ServerOptions;
use crate::server::registry::ServerEntry;
use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
#[cfg(test)]
use std::io::Cursor;
use std::io::{Read, Seek, Write};
use std::path::{Component, Path, PathBuf};
use tar::Archive;
use tokio::io::AsyncWriteExt;
use tokio::time;
use xz2::read::XzDecoder;
use zip::ZipArchive;

const GITHUB_API_VERSION_HEADER: &str = "X-GitHub-Api-Version";
const GITHUB_API_VERSION: &str = "2022-11-28";
const USER_AGENT_VALUE: &str = "lsp-io";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseTarget {
    WindowsX64,
    WindowsArm64,
    LinuxX64Gnu,
    LinuxArm64Gnu,
    MacosX64,
    MacosArm64,
}

#[derive(Debug, Clone, Copy)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
    TarXz,
    Tgz,
    GzipBinary,
    RawBinary,
}

#[derive(Debug)]
pub enum ReleaseSelector {
    Latest,
    Tag(&'static str),
}

#[derive(Debug)]
pub struct ReleaseAssetSpec {
    pub target: ReleaseTarget,
    pub asset_name: &'static str,
    pub archive_format: ArchiveFormat,
    pub binary_path: &'static str,
    pub sha256: Option<&'static str>,
}

#[derive(Debug)]
pub struct GithubReleaseSpec {
    pub owner: &'static str,
    pub repo: &'static str,
    pub selector: ReleaseSelector,
    pub max_size_bytes: u64,
    pub max_extract_size_bytes: u64,
    pub install_warning: Option<&'static str>,
    pub assets: &'static [ReleaseAssetSpec],
}

#[derive(Debug, Clone)]
pub struct GithubReleaseInstallInfo {
    pub binary_path: PathBuf,
    pub release_tag: String,
    pub asset_name: String,
    pub asset_digest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubReleaseResponse {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
    digest: Option<String>,
}

#[derive(Debug)]
struct ResolvedGithubAsset<'a> {
    asset_spec: &'a ReleaseAssetSpec,
    release_tag: String,
    asset_name: String,
    download_url: String,
    digest: Option<String>,
}

#[derive(Debug)]
struct DownloadedAsset {
    path: PathBuf,
    sha256: String,
}

pub async fn install_github_release_archive(
    entry: &ServerEntry,
    spec: &GithubReleaseSpec,
    root: &Path,
    options: &ServerOptions,
    progress: &dyn ProgressHandler,
) -> Result<GithubReleaseInstallInfo> {
    let target = current_release_target()?;
    let asset_spec = asset_for_target(spec, target)?;

    progress.on_event(ProgressEvent::InstallOutput {
        server: entry.name.to_string(),
        line: format!(
            "Resolving GitHub release {}/{} for {}",
            spec.owner, spec.repo, asset_spec.asset_name
        ),
    });

    let release = fetch_release(spec, options).await?;
    let asset = resolve_asset_from_release(spec, &release, target)?;

    progress.on_event(ProgressEvent::InstallOutput {
        server: entry.name.to_string(),
        line: format!("Downloading {}", asset.asset_name),
    });

    let extract_root = root.join("github-release");
    fs::create_dir_all(&extract_root)
        .with_context(|| format!("Failed to create {}", extract_root.display()))?;

    let download_root = root.join("downloads");
    fs::create_dir_all(&download_root)
        .with_context(|| format!("Failed to create {}", download_root.display()))?;
    let download_path = safe_join(&download_root, Path::new(&asset.asset_name))?;
    let downloaded = download_asset(&asset, spec, options, &download_path).await?;
    verify_download_digest(
        &downloaded.sha256,
        asset.asset_spec.sha256,
        asset.digest.as_deref(),
    )
    .with_context(|| format!("Failed to verify {}", asset.asset_name))?;

    extract_archive_from_file(
        &downloaded.path,
        asset.asset_spec.archive_format,
        &extract_root,
        spec.max_extract_size_bytes,
        Path::new(asset.asset_spec.binary_path),
    )
    .with_context(|| format!("Failed to extract {}", asset.asset_name))?;
    fs::remove_file(&downloaded.path).ok();

    let binary_path = binary_path_in_root(spec, &extract_root).with_context(|| {
        format!(
            "No archive binary mapping for current platform in {}",
            entry.name
        )
    })?;
    if !binary_path.is_file() {
        bail!(
            "{} archive extracted but expected binary was not found at {}",
            entry.name,
            binary_path.display()
        );
    }
    ensure_executable(&binary_path)?;

    Ok(GithubReleaseInstallInfo {
        binary_path,
        release_tag: asset.release_tag,
        asset_name: asset.asset_name,
        asset_digest: asset.digest,
    })
}

pub fn is_supported_on_current_platform(spec: &GithubReleaseSpec) -> bool {
    asset_for_current_platform(spec).is_ok()
}

pub fn binary_path_in_root(spec: &GithubReleaseSpec, root: &Path) -> Option<PathBuf> {
    let asset = asset_for_current_platform(spec).ok()?;
    safe_join(root, Path::new(asset.binary_path)).ok()
}

pub fn asset_for_current_platform(spec: &GithubReleaseSpec) -> Result<&ReleaseAssetSpec> {
    let target = current_release_target()?;
    asset_for_target(spec, target)
}

fn current_release_target() -> Result<ReleaseTarget> {
    release_target_for_parts(std::env::consts::OS, std::env::consts::ARCH)
        .ok_or_else(|| anyhow::anyhow!("No managed GitHub release target for this platform"))
}

fn release_target_for_parts(os: &str, arch: &str) -> Option<ReleaseTarget> {
    match (os, arch) {
        ("windows", "x86_64") => Some(ReleaseTarget::WindowsX64),
        ("windows", "aarch64") => Some(ReleaseTarget::WindowsArm64),
        ("linux", "x86_64") => Some(ReleaseTarget::LinuxX64Gnu),
        ("linux", "aarch64") => Some(ReleaseTarget::LinuxArm64Gnu),
        ("macos", "x86_64") => Some(ReleaseTarget::MacosX64),
        ("macos", "aarch64") => Some(ReleaseTarget::MacosArm64),
        _ => None,
    }
}

fn asset_for_target(spec: &GithubReleaseSpec, target: ReleaseTarget) -> Result<&ReleaseAssetSpec> {
    spec.assets
        .iter()
        .find(|asset| asset.target == target)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No managed GitHub release asset is configured for this platform in {}/{}",
                spec.owner,
                spec.repo
            )
        })
}

fn release_api_url(spec: &GithubReleaseSpec) -> String {
    match spec.selector {
        ReleaseSelector::Latest => format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            spec.owner, spec.repo
        ),
        ReleaseSelector::Tag(tag) => format!(
            "https://api.github.com/repos/{}/{}/releases/tags/{}",
            spec.owner, spec.repo, tag
        ),
    }
}

async fn fetch_release(
    spec: &GithubReleaseSpec,
    options: &ServerOptions,
) -> Result<GithubReleaseResponse> {
    let client = reqwest::Client::builder()
        .default_headers(github_headers()?)
        .build()?;
    let response = time::timeout(options.timeout, client.get(release_api_url(spec)).send())
        .await
        .with_context(|| {
            format!(
                "GitHub release lookup for {}/{} timed out after {} seconds",
                spec.owner,
                spec.repo,
                options.timeout.as_secs()
            )
        })?
        .with_context(|| {
            format!(
                "Failed to query GitHub release for {}/{}",
                spec.owner, spec.repo
            )
        })?;

    let status = response.status();
    if !status.is_success() {
        bail!(
            "GitHub release lookup for {}/{} failed with HTTP {}",
            spec.owner,
            spec.repo,
            status
        );
    }

    response.json().await.with_context(|| {
        format!(
            "Failed to parse GitHub release response for {}/{}",
            spec.owner, spec.repo
        )
    })
}

fn resolve_asset_from_release<'a>(
    spec: &'a GithubReleaseSpec,
    release: &GithubReleaseResponse,
    target: ReleaseTarget,
) -> Result<ResolvedGithubAsset<'a>> {
    let asset_spec = asset_for_target(spec, target)?;
    let asset = release
        .assets
        .iter()
        .find(|candidate| candidate.name == asset_spec.asset_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Release {} for {}/{} does not contain expected asset {}",
                release.tag_name,
                spec.owner,
                spec.repo,
                asset_spec.asset_name
            )
        })?;

    if asset.size > spec.max_size_bytes {
        bail!(
            "{} is {} bytes, above the configured {} byte limit",
            asset.name,
            asset.size,
            spec.max_size_bytes
        );
    }

    Ok(ResolvedGithubAsset {
        asset_spec,
        release_tag: release.tag_name.clone(),
        asset_name: asset.name.clone(),
        download_url: asset.browser_download_url.clone(),
        digest: asset.digest.clone(),
    })
}

async fn download_asset(
    asset: &ResolvedGithubAsset<'_>,
    spec: &GithubReleaseSpec,
    options: &ServerOptions,
    destination: &Path,
) -> Result<DownloadedAsset> {
    time::timeout(
        options.timeout,
        download_asset_inner(asset, spec, destination),
    )
    .await
    .with_context(|| {
        format!(
            "Download of {} timed out after {} seconds",
            asset.asset_name,
            options.timeout.as_secs()
        )
    })?
}

async fn download_asset_inner(
    asset: &ResolvedGithubAsset<'_>,
    spec: &GithubReleaseSpec,
    destination: &Path,
) -> Result<DownloadedAsset> {
    let client = reqwest::Client::builder()
        .default_headers(github_headers()?)
        .build()?;
    let mut response = client
        .get(&asset.download_url)
        .send()
        .await
        .with_context(|| format!("Failed to download {}", asset.asset_name))?;

    let status = response.status();
    if !status.is_success() {
        bail!(
            "Download of {} failed with HTTP {}",
            asset.asset_name,
            status
        );
    }

    if let Some(content_length) = response.content_length() {
        if content_length > spec.max_size_bytes {
            bail!(
                "{} reports {} bytes, above the configured {} byte limit",
                asset.asset_name,
                content_length,
                spec.max_size_bytes
            );
        }
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = tokio::fs::File::create(destination)
        .await
        .with_context(|| format!("Failed to create {}", destination.display()))?;
    let mut written = 0_u64;
    let mut hasher = Sha256::new();
    while let Some(chunk) = response.chunk().await? {
        let chunk_len = chunk.len() as u64;
        if written + chunk_len > spec.max_size_bytes {
            bail!(
                "{} exceeded the configured {} byte limit while downloading",
                asset.asset_name,
                spec.max_size_bytes
            );
        }
        file.write_all(&chunk)
            .await
            .with_context(|| format!("Failed to write {}", destination.display()))?;
        hasher.update(&chunk);
        written += chunk_len;
    }
    file.flush()
        .await
        .with_context(|| format!("Failed to flush {}", destination.display()))?;

    let digest = hasher.finalize();
    Ok(DownloadedAsset {
        path: destination.to_path_buf(),
        sha256: digest.iter().map(|byte| format!("{byte:02x}")).collect(),
    })
}

fn github_headers() -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        GITHUB_API_VERSION_HEADER,
        HeaderValue::from_static(GITHUB_API_VERSION),
    );
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        let value = HeaderValue::from_str(&format!("Bearer {token}"))
            .context("GITHUB_TOKEN contains characters that are invalid in an HTTP header")?;
        headers.insert(AUTHORIZATION, value);
    }
    Ok(headers)
}

#[cfg(test)]
fn verify_download(
    bytes: &[u8],
    registry_sha256: Option<&str>,
    api_digest: Option<&str>,
) -> Result<()> {
    let actual = sha256_hex(bytes);
    verify_download_digest(&actual, registry_sha256, api_digest)
}

fn verify_download_digest(
    actual: &str,
    registry_sha256: Option<&str>,
    api_digest: Option<&str>,
) -> Result<()> {
    if let Some(expected) = registry_sha256 {
        verify_sha256_value("registry", expected, actual)?;
    }

    if let Some(digest) = api_digest {
        if let Some(expected) = digest.strip_prefix("sha256:") {
            verify_sha256_value("GitHub API", expected, actual)?;
        }
    }

    Ok(())
}

fn verify_sha256_value(source: &str, expected: &str, actual: &str) -> Result<()> {
    if !expected.eq_ignore_ascii_case(actual) {
        bail!("{source} sha256 mismatch: expected {expected}, got {actual}");
    }
    Ok(())
}

#[cfg(test)]
fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn extract_archive_from_file(
    archive_path: &Path,
    format: ArchiveFormat,
    root: &Path,
    max_extract_size_bytes: u64,
    binary_path: &Path,
) -> Result<()> {
    match format {
        ArchiveFormat::Zip => extract_zip(
            fs::File::open(archive_path)
                .with_context(|| format!("Failed to open {}", archive_path.display()))?,
            root,
            max_extract_size_bytes,
        ),
        ArchiveFormat::TarGz | ArchiveFormat::Tgz => extract_tar_gz(
            fs::File::open(archive_path)
                .with_context(|| format!("Failed to open {}", archive_path.display()))?,
            root,
            max_extract_size_bytes,
        ),
        ArchiveFormat::TarXz => extract_tar_xz(
            fs::File::open(archive_path)
                .with_context(|| format!("Failed to open {}", archive_path.display()))?,
            root,
            max_extract_size_bytes,
        ),
        ArchiveFormat::GzipBinary => {
            let mut file = fs::File::open(archive_path)
                .with_context(|| format!("Failed to open {}", archive_path.display()))?;
            extract_gzip_binary(&mut file, root, max_extract_size_bytes, binary_path)
        }
        ArchiveFormat::RawBinary => {
            let mut file = fs::File::open(archive_path)
                .with_context(|| format!("Failed to open {}", archive_path.display()))?;
            extract_raw_binary(&mut file, root, max_extract_size_bytes, binary_path)
        }
    }
}

#[cfg(test)]
fn extract_archive_from_bytes(
    bytes: &[u8],
    format: ArchiveFormat,
    root: &Path,
    max_extract_size_bytes: u64,
    binary_path: &Path,
) -> Result<()> {
    match format {
        ArchiveFormat::Zip => extract_zip(Cursor::new(bytes), root, max_extract_size_bytes),
        ArchiveFormat::TarGz | ArchiveFormat::Tgz => {
            extract_tar_gz(Cursor::new(bytes), root, max_extract_size_bytes)
        }
        ArchiveFormat::TarXz => extract_tar_xz(Cursor::new(bytes), root, max_extract_size_bytes),
        ArchiveFormat::GzipBinary => {
            let mut cursor = Cursor::new(bytes);
            extract_gzip_binary(&mut cursor, root, max_extract_size_bytes, binary_path)
        }
        ArchiveFormat::RawBinary => {
            let mut cursor = Cursor::new(bytes);
            extract_raw_binary(&mut cursor, root, max_extract_size_bytes, binary_path)
        }
    }
}

fn extract_zip<R: Read + Seek>(reader: R, root: &Path, max_extract_size_bytes: u64) -> Result<()> {
    let mut archive = ZipArchive::new(reader).context("Invalid zip archive")?;
    let mut remaining_extract_bytes = max_extract_size_bytes;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        let out_path = safe_join(root, Path::new(file.name()))?;
        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out_file = fs::File::create(&out_path)?;
        copy_with_extract_limit(
            &mut file,
            &mut out_file,
            &mut remaining_extract_bytes,
            &out_path,
        )?;

        #[cfg(unix)]
        if let Some(mode) = file.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
        }
    }

    Ok(())
}

fn extract_tar_gz<R: Read>(reader: R, root: &Path, max_extract_size_bytes: u64) -> Result<()> {
    let decoder = GzDecoder::new(reader);
    extract_tar_stream(decoder, root, max_extract_size_bytes)
}

fn extract_tar_xz<R: Read>(reader: R, root: &Path, max_extract_size_bytes: u64) -> Result<()> {
    let decoder = XzDecoder::new(reader);
    extract_tar_stream(decoder, root, max_extract_size_bytes)
}

fn extract_tar_stream<R: Read>(reader: R, root: &Path, max_extract_size_bytes: u64) -> Result<()> {
    let mut archive = Archive::new(reader);
    let mut remaining_extract_bytes = max_extract_size_bytes;

    for entry in archive.entries().context("Invalid tar archive")? {
        let mut entry = entry?;
        let relative = entry.path()?.to_path_buf();
        let out_path = safe_join(root, &relative)?;
        let entry_type = entry.header().entry_type();

        if entry_type.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else if entry_type.is_file() {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            copy_with_extract_limit(
                &mut entry,
                &mut out_file,
                &mut remaining_extract_bytes,
                &out_path,
            )?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(mode) = entry.header().mode() {
                    fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
                }
            }
        } else if entry_type.is_symlink() || entry_type.is_hard_link() {
            // LLVM and similar toolchain archives include many library links that
            // are not needed to stage the configured LSP binary. Skipping them is
            // safer and more portable than creating archive-controlled links.
            continue;
        } else {
            bail!("Unsupported tar entry type for {}", relative.display());
        }
    }

    Ok(())
}

fn extract_gzip_binary(
    reader: &mut impl Read,
    root: &Path,
    max_extract_size_bytes: u64,
    binary_path: &Path,
) -> Result<()> {
    let mut decoder = GzDecoder::new(reader);
    extract_single_binary(&mut decoder, root, max_extract_size_bytes, binary_path)
}

fn extract_raw_binary(
    reader: &mut impl Read,
    root: &Path,
    max_extract_size_bytes: u64,
    binary_path: &Path,
) -> Result<()> {
    extract_single_binary(reader, root, max_extract_size_bytes, binary_path)
}

fn extract_single_binary<R: Read>(
    reader: &mut R,
    root: &Path,
    max_extract_size_bytes: u64,
    binary_path: &Path,
) -> Result<()> {
    let out_path = safe_join(root, binary_path)?;
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut out_file = fs::File::create(&out_path)?;
    let mut remaining_extract_bytes = max_extract_size_bytes;
    copy_with_extract_limit(
        reader,
        &mut out_file,
        &mut remaining_extract_bytes,
        &out_path,
    )?;
    ensure_executable(&out_path)?;
    Ok(())
}

fn copy_with_extract_limit<R: Read>(
    reader: &mut R,
    writer: &mut fs::File,
    remaining_extract_bytes: &mut u64,
    path: &Path,
) -> Result<()> {
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Ok(());
        }
        let read = read as u64;
        if read > *remaining_extract_bytes {
            bail!(
                "Archive extraction exceeded the configured {} byte limit while writing {}",
                *remaining_extract_bytes,
                path.display()
            );
        }
        writer.write_all(&buffer[..read as usize])?;
        *remaining_extract_bytes -= read;
    }
}

fn safe_join(root: &Path, relative: &Path) -> Result<PathBuf> {
    let mut sanitized = PathBuf::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => sanitized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("Archive entry escapes install root: {}", relative.display())
            }
        }
    }

    if sanitized.as_os_str().is_empty() {
        bail!("Archive entry has an empty path");
    }

    Ok(root.join(sanitized))
}

fn ensure_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path)?;
        let mut permissions = metadata.permissions();
        let mode = permissions.mode();
        if mode & 0o100 == 0 {
            permissions.set_mode(mode | 0o755);
            fs::set_permissions(path, permissions)?;
        }
    }

    #[cfg(not(unix))]
    {
        let _ = path;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::ServerOptions;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;
    use std::time::Duration;
    use tar::{Builder, EntryType, Header};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use xz2::write::XzEncoder;
    use zip::write::SimpleFileOptions;

    const TEST_ASSETS: &[ReleaseAssetSpec] = &[
        ReleaseAssetSpec {
            target: ReleaseTarget::WindowsX64,
            asset_name: "tool-windows-x64.tar.gz",
            archive_format: ArchiveFormat::TarGz,
            binary_path: "bin/tool.exe",
            sha256: None,
        },
        ReleaseAssetSpec {
            target: ReleaseTarget::LinuxX64Gnu,
            asset_name: "tool-linux-x64.tar.gz",
            archive_format: ArchiveFormat::TarGz,
            binary_path: "bin/tool",
            sha256: None,
        },
        ReleaseAssetSpec {
            target: ReleaseTarget::MacosArm64,
            asset_name: "tool-darwin-arm64.zip",
            archive_format: ArchiveFormat::Zip,
            binary_path: "bin/tool",
            sha256: None,
        },
    ];

    const TEST_SPEC: GithubReleaseSpec = GithubReleaseSpec {
        owner: "owner",
        repo: "repo",
        selector: ReleaseSelector::Tag("v1.0.0"),
        max_size_bytes: 1024 * 1024,
        max_extract_size_bytes: 1024 * 1024,
        install_warning: None,
        assets: TEST_ASSETS,
    };

    #[test]
    fn release_target_selection_maps_common_platforms() {
        assert_eq!(
            release_target_for_parts("windows", "x86_64"),
            Some(ReleaseTarget::WindowsX64)
        );
        assert_eq!(
            release_target_for_parts("linux", "x86_64"),
            Some(ReleaseTarget::LinuxX64Gnu)
        );
        assert_eq!(
            release_target_for_parts("macos", "aarch64"),
            Some(ReleaseTarget::MacosArm64)
        );
        assert_eq!(release_target_for_parts("freebsd", "x86_64"), None);
    }

    #[test]
    fn release_url_uses_exact_tag_or_latest() {
        assert_eq!(
            release_api_url(&TEST_SPEC),
            "https://api.github.com/repos/owner/repo/releases/tags/v1.0.0"
        );
        let latest = GithubReleaseSpec {
            selector: ReleaseSelector::Latest,
            ..TEST_SPEC
        };
        assert_eq!(
            release_api_url(&latest),
            "https://api.github.com/repos/owner/repo/releases/latest"
        );
    }

    #[test]
    fn resolve_asset_matches_exact_filename() {
        let release = GithubReleaseResponse {
            tag_name: "v1.0.0".to_string(),
            assets: vec![
                GithubAsset {
                    name: "prefix-tool-linux-x64.tar.gz".to_string(),
                    size: 10,
                    browser_download_url: "https://example.com/wrong".to_string(),
                    digest: None,
                },
                GithubAsset {
                    name: "tool-linux-x64.tar.gz".to_string(),
                    size: 10,
                    browser_download_url: "https://example.com/right".to_string(),
                    digest: None,
                },
            ],
        };

        let resolved =
            resolve_asset_from_release(&TEST_SPEC, &release, ReleaseTarget::LinuxX64Gnu).unwrap();

        assert_eq!(resolved.download_url, "https://example.com/right");
    }

    #[test]
    fn resolve_asset_rejects_oversized_asset() {
        let spec = GithubReleaseSpec {
            max_size_bytes: 5,
            ..TEST_SPEC
        };
        let release = GithubReleaseResponse {
            tag_name: "v1.0.0".to_string(),
            assets: vec![GithubAsset {
                name: "tool-linux-x64.tar.gz".to_string(),
                size: 10,
                browser_download_url: "https://example.com/tool".to_string(),
                digest: None,
            }],
        };

        let result = resolve_asset_from_release(&spec, &release, ReleaseTarget::LinuxX64Gnu);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("above"));
    }

    #[tokio::test]
    async fn download_timeout_covers_body_stream() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 1024];
            let _ = socket.read(&mut request).await;
            let _ = socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\n")
                .await;
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = socket.write_all(b"slow").await;
        });
        let asset = ResolvedGithubAsset {
            asset_spec: &TEST_ASSETS[1],
            release_tag: "v1.0.0".to_string(),
            asset_name: "tool-linux-x64.tar.gz".to_string(),
            download_url: format!("http://{address}/tool-linux-x64.tar.gz"),
            digest: None,
        };
        let options = ServerOptions {
            timeout: Duration::from_millis(50),
            ..ServerOptions::default()
        };

        let dir = tempfile::tempdir().unwrap();
        let result = download_asset(
            &asset,
            &TEST_SPEC,
            &options,
            &dir.path().join("download.tar.gz"),
        )
        .await;
        server.abort();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[test]
    fn registry_sha256_mismatch_is_rejected() {
        let result = verify_download(b"hello", Some("0000"), None);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("registry sha256 mismatch")
        );
    }

    #[test]
    fn api_digest_mismatch_is_rejected() {
        let result = verify_download(b"hello", None, Some("sha256:0000"));

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("GitHub API sha256 mismatch")
        );
    }

    #[test]
    fn zip_archive_extracts_nested_binary() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = zip_bytes(&[("nested/tool.exe", b"binary".as_slice())]);

        extract_archive_from_bytes(
            &bytes,
            ArchiveFormat::Zip,
            dir.path(),
            1024,
            Path::new("nested/tool.exe"),
        )
        .unwrap();

        assert_eq!(
            fs::read(dir.path().join("nested/tool.exe")).unwrap(),
            b"binary"
        );
    }

    #[test]
    fn tar_gz_archive_extracts_nested_binary() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = tar_gz_bytes(&[("nested/tool", b"binary".as_slice())]);

        extract_archive_from_bytes(
            &bytes,
            ArchiveFormat::TarGz,
            dir.path(),
            1024,
            Path::new("nested/tool"),
        )
        .unwrap();

        assert_eq!(fs::read(dir.path().join("nested/tool")).unwrap(), b"binary");
    }

    #[test]
    fn tar_xz_archive_extracts_nested_binary() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = tar_xz_bytes(&[("nested/tool", b"binary".as_slice())]);

        extract_archive_from_bytes(
            &bytes,
            ArchiveFormat::TarXz,
            dir.path(),
            1024,
            Path::new("nested/tool"),
        )
        .unwrap();

        assert_eq!(fs::read(dir.path().join("nested/tool")).unwrap(), b"binary");
    }

    #[test]
    fn tar_gz_archive_skips_links_and_extracts_binary() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = tar_gz_bytes_with_link();

        extract_archive_from_bytes(
            &bytes,
            ArchiveFormat::TarGz,
            dir.path(),
            1024,
            Path::new("nested/tool"),
        )
        .unwrap();

        assert_eq!(fs::read(dir.path().join("nested/tool")).unwrap(), b"binary");
        assert!(!dir.path().join("nested/libtool.so").exists());
    }

    #[test]
    fn gzip_binary_extracts_to_configured_binary_path() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = gzip_binary_bytes(b"binary");

        extract_archive_from_bytes(
            &bytes,
            ArchiveFormat::GzipBinary,
            dir.path(),
            1024,
            Path::new("tool/bin/server"),
        )
        .unwrap();

        assert_eq!(
            fs::read(dir.path().join("tool/bin/server")).unwrap(),
            b"binary"
        );
    }

    #[test]
    fn raw_binary_extracts_to_configured_binary_path() {
        let dir = tempfile::tempdir().unwrap();

        extract_archive_from_bytes(
            b"binary",
            ArchiveFormat::RawBinary,
            dir.path(),
            1024,
            Path::new("tool.exe"),
        )
        .unwrap();

        assert_eq!(fs::read(dir.path().join("tool.exe")).unwrap(), b"binary");
    }

    #[test]
    fn zip_archive_rejects_extracted_size_limit() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = zip_bytes(&[("nested/tool.exe", b"too-large".as_slice())]);

        let result = extract_archive_from_bytes(
            &bytes,
            ArchiveFormat::Zip,
            dir.path(),
            4,
            Path::new("nested/tool.exe"),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeded"));
    }

    #[test]
    fn tar_gz_archive_rejects_extracted_size_limit() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = tar_gz_bytes(&[("nested/tool", b"too-large".as_slice())]);

        let result = extract_archive_from_bytes(
            &bytes,
            ArchiveFormat::TarGz,
            dir.path(),
            4,
            Path::new("nested/tool"),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeded"));
    }

    #[test]
    fn zip_archive_rejects_parent_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = zip_bytes(&[("../escape.exe", b"bad".as_slice())]);

        let result = extract_archive_from_bytes(
            &bytes,
            ArchiveFormat::Zip,
            dir.path(),
            1024,
            Path::new("escape.exe"),
        );

        assert!(result.is_err());
        assert!(!dir.path().join("..").join("escape.exe").exists());
    }

    #[test]
    fn tar_gz_archive_rejects_parent_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = malicious_tar_gz_bytes("../escape", b"bad");

        let result = extract_archive_from_bytes(
            &bytes,
            ArchiveFormat::TarGz,
            dir.path(),
            1024,
            Path::new("escape"),
        );

        assert!(result.is_err());
        assert!(!dir.path().join("..").join("escape").exists());
    }

    fn zip_bytes(files: &[(&str, &[u8])]) -> Vec<u8> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(cursor);
        for (path, bytes) in files {
            writer
                .start_file(*path, SimpleFileOptions::default())
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn tar_gz_bytes(files: &[(&str, &[u8])]) -> Vec<u8> {
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut builder = Builder::new(encoder);
        for (path, bytes) in files {
            let mut header = Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder.append_data(&mut header, *path, *bytes).unwrap();
        }
        builder.finish().unwrap();
        builder.into_inner().unwrap().finish().unwrap()
    }

    fn tar_gz_bytes_with_link() -> Vec<u8> {
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut builder = Builder::new(encoder);

        let mut header = Header::new_gnu();
        header.set_entry_type(EntryType::Symlink);
        header.set_size(0);
        builder
            .append_link(&mut header, "nested/libtool.so", "libtool.so.1")
            .unwrap();

        let mut binary_header = Header::new_gnu();
        binary_header.set_size(6);
        binary_header.set_mode(0o755);
        binary_header.set_cksum();
        builder
            .append_data(&mut binary_header, "nested/tool", b"binary".as_slice())
            .unwrap();

        builder.finish().unwrap();
        builder.into_inner().unwrap().finish().unwrap()
    }

    fn tar_xz_bytes(files: &[(&str, &[u8])]) -> Vec<u8> {
        let encoder = XzEncoder::new(Vec::new(), 6);
        let mut builder = Builder::new(encoder);
        for (path, bytes) in files {
            let mut header = Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder.append_data(&mut header, *path, *bytes).unwrap();
        }
        builder.finish().unwrap();
        builder.into_inner().unwrap().finish().unwrap()
    }

    fn gzip_binary_bytes(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    fn malicious_tar_gz_bytes(path: &str, bytes: &[u8]) -> Vec<u8> {
        let mut builder = Builder::new(Vec::new());
        let mut header = Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder
            .append_data(&mut header, "aa/escape", bytes)
            .unwrap();
        builder.finish().unwrap();
        let mut tar = builder.into_inner().unwrap();

        tar[0..100].fill(0);
        tar[0..path.len()].copy_from_slice(path.as_bytes());
        tar[148..156].fill(b' ');
        let checksum: u32 = tar.iter().map(|byte| *byte as u32).sum();
        let encoded = format!("{checksum:06o}\0 ");
        tar[148..156].copy_from_slice(encoded.as_bytes());

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&tar).unwrap();
        encoder.finish().unwrap()
    }
}
