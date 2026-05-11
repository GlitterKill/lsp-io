# Managed GitHub Release Archive Installer Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe managed installer for language servers distributed as GitHub release archives, then promote Ada Language Server from manual guidance to managed install.

**Architecture:** Extend the existing managed install pipeline instead of creating a parallel path. Registry entries will describe GitHub owner/repo, target-specific asset names, archive format, expected binary path, and size/checksum policy; the installer will resolve/download/extract into the existing staging directory and then reuse the existing manifest-backed promotion/removal flow.

**Tech Stack:** Rust 2024, Tokio, reqwest with rustls, zip, tar, flate2, sha2, existing LSP-IO staging/manifest cache, GitHub REST Releases API.

---

## Research Notes

- GitHub releases expose `GET /repos/{owner}/{repo}/releases/latest` and `GET /repos/{owner}/{repo}/releases/tags/{tag}` responses that include release assets, asset names, sizes, `browser_download_url`, and optional `digest` metadata.
- GitHub release assets can be downloaded by fetching `browser_download_url`, or by requesting the asset API URL with `Accept: application/octet-stream`; API clients must handle either `200` or `302`.
- Ada Language Server publishes official platform release archives. Mason already maps those archives per target and nested binary path, so ALS is a good first promotion candidate.

Sources:
- https://docs.github.com/en/rest/releases/releases
- https://docs.github.com/en/rest/releases/assets
- https://github.com/AdaCore/ada_language_server#install
- https://raw.githubusercontent.com/mason-org/mason-registry/main/packages/ada-language-server/package.yaml

## Design Decisions

### Decision 1: Generic Archive Installer Instead Of Per-Server Scripts

Pros:
- One safety-reviewed code path for ALS, LuaLS, Marksman, Taplo, texlab, tinymist, Verible, and similar archive-backed tools.
- Keeps removal safe because all installs still use the existing LSP-IO manifest.
- Makes the Install/Guide distinction more honest without hand-writing brittle installers.

Cons:
- Requires careful platform matching, archive extraction, and path traversal protection.
- Adds HTTP and archive dependencies to `lsp-io-core`.

### Decision 2: Prefer Pinned Tags For First Promoted Entries

Pros:
- Reproducible installs and testable asset names.
- Easier checksum verification and fewer surprise upstream layout changes.

Cons:
- Registry updates are needed to pick up new releases until an update workflow exists.
- Users may expect "latest" semantics because package-manager installers currently use latest for several entries.

Implementation rule: support `Latest` in the type model, but promote ALS with an exact current tag first. Add latest-update behavior later.

### Decision 3: Use GitHub API Metadata, Then Download `browser_download_url`

Pros:
- API response gives asset size, name, release tag, and `digest` when available.
- `browser_download_url` avoids custom redirect handling for the binary asset API in v1.

Cons:
- Public unauthenticated API calls can hit rate limits.
- Some assets may lack digest metadata, so registry-provided checksums still matter.

Implementation rule: use optional `GITHUB_TOKEN` for higher rate limits when present, never log it, and keep public unauthenticated installs working.

## File Map

- Modify `Cargo.toml`: add workspace dependencies for HTTP, archive extraction, and checksums.
- Modify `crates/lsp-io-core/Cargo.toml`: consume new workspace dependencies.
- Create `crates/lsp-io-core/src/server/github_release.rs`: GitHub release metadata structs, target selection, API client, download, digest verification, safe archive extraction.
- Modify `crates/lsp-io-core/src/server/mod.rs`: expose the new internal module.
- Modify `crates/lsp-io-core/src/server/registry.rs`: add `InstallMethod::GithubReleaseArchive`, a `github_release_entry` helper, and convert ALS.
- Modify `crates/lsp-io-core/src/server/install.rs`: dispatch the new install method, compute archive-managed binary paths, and include archive metadata in manifests.
- Modify `README.md`: document GitHub archive managed installs, checksum/manifest safety, and `GITHUB_TOKEN`.
- Modify `CHANGELOG.md`: add the managed GitHub archive installer and ALS promotion.
- Optional modify `gui/src/components/Dashboard.ts`: update Vite mock rows so ALS shows `github release` and `Install`.

## Chunk 1: Data Model And Platform Matching

### Task 1: Add installer metadata types

**Files:**
- Create: `crates/lsp-io-core/src/server/github_release.rs`
- Modify: `crates/lsp-io-core/src/server/mod.rs`
- Modify: `crates/lsp-io-core/src/server/registry.rs`

- [ ] **Step 1: Write failing platform-selection tests**

Add tests in `github_release.rs` for:
- Windows x64 selects `ReleaseTarget::WindowsX64`
- Linux x64 selects `ReleaseTarget::LinuxX64Gnu`
- macOS arm64 selects `ReleaseTarget::MacosArm64`
- unsupported target returns a clear error

- [ ] **Step 2: Implement metadata structs**

Use a small registry-facing model:

```rust
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
    Tgz,
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
    pub assets: &'static [ReleaseAssetSpec],
}
```

- [ ] **Step 3: Wire module visibility**

Add `pub mod github_release;` to `crates/lsp-io-core/src/server/mod.rs`.

- [ ] **Step 4: Run focused tests**

Run: `cargo test -p lsp-io-core github_release`

Expected: platform-selection tests pass.

## Chunk 2: Archive Extraction Safety

### Task 2: Implement safe extraction helpers

**Files:**
- Modify: `crates/lsp-io-core/src/server/github_release.rs`
- Modify: `crates/lsp-io-core/Cargo.toml`
- Modify: `Cargo.toml`

- [ ] **Step 1: Add failing extraction tests**

Add tempdir tests for:
- `.zip` extracts a nested executable to the expected relative path.
- `.tar.gz` extracts a nested executable to the expected relative path.
- zip-slip paths like `../escape.exe` are rejected.
- tar entries with absolute paths or parent traversal are rejected.

- [ ] **Step 2: Add dependencies**

Add workspace dependencies:

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
sha2 = "0.10"
zip = "2"
tar = "0.4"
flate2 = "1"
```

If `zip = "2"` conflicts during implementation, use the latest compatible major from `cargo add zip` and let the lockfile choose the concrete version.

- [ ] **Step 3: Implement traversal-safe path handling**

Add a helper that canonicalizes the extraction root and rejects entries whose normalized output path would leave it. Do not rely on archive crate convenience extraction directly.

- [ ] **Step 4: Preserve executable permissions on Unix**

When extracting the selected binary on Unix, set owner-executable bits if the archive entry metadata does not preserve them.

- [ ] **Step 5: Run focused tests**

Run: `cargo test -p lsp-io-core archive`

Expected: extraction tests pass and traversal tests fail closed.

## Chunk 3: GitHub Release Resolution And Download

### Task 3: Add API client and download verification

**Files:**
- Modify: `crates/lsp-io-core/src/server/github_release.rs`

- [ ] **Step 1: Add failing tests around pure resolution**

Use JSON fixtures or constructed structs to test:
- exact tag endpoint is chosen when selector is `Tag`.
- latest endpoint is chosen when selector is `Latest`.
- asset selection is exact by filename, not substring.
- asset size above `max_size_bytes` is rejected.
- `sha256:` API digest mismatch is rejected.
- registry `sha256` mismatch is rejected.

- [ ] **Step 2: Implement GitHub API structs**

Only deserialize fields needed by the installer:

```rust
#[derive(Deserialize)]
struct GithubReleaseResponse {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
    digest: Option<String>,
}
```

- [ ] **Step 3: Implement HTTP client**

Use:
- `User-Agent: lsp-io`
- `Accept: application/vnd.github+json`
- `X-GitHub-Api-Version` as one constant
- optional `Authorization: Bearer $GITHUB_TOKEN` only when the env var exists

- [ ] **Step 4: Implement download limit and checksum verification**

Do not load unbounded responses. Reject when:
- `Content-Length` exceeds `max_size_bytes`
- streamed bytes exceed `max_size_bytes`
- registry checksum exists and does not match
- API digest exists and does not match

- [ ] **Step 5: Run tests**

Run: `cargo test -p lsp-io-core github_release`

Expected: resolver and checksum tests pass without live network.

## Chunk 4: Install Pipeline Integration

### Task 4: Add `GithubReleaseArchive` to `InstallMethod`

**Files:**
- Modify: `crates/lsp-io-core/src/server/registry.rs`
- Modify: `crates/lsp-io-core/src/server/install.rs`

- [ ] **Step 1: Add failing registry/install path tests**

Add tests for:
- `InstallMethod::GithubReleaseArchive` is managed.
- label is `github release`.
- `binary_path_under` returns the current target's archive binary path under `github-release`.
- unsupported current target reports non-installable if this method lacks a matching asset.

- [ ] **Step 2: Extend the enum**

Add:

```rust
GithubReleaseArchive { spec: &'static GithubReleaseSpec },
```

- [ ] **Step 3: Update method helpers**

Update:
- `is_managed`
- `label`
- `manual_instructions`
- add `is_supported_on_current_platform`

Use `is_supported_on_current_platform` for `ServerStatusInfo.can_install`, not only `is_managed`.

- [ ] **Step 4: Dispatch installer**

In `install_server_with_options`, add a match arm:

```rust
InstallMethod::GithubReleaseArchive { spec } => {
    github_release::install_github_release_archive(entry, spec, &stage_root, options, progress).await
}
```

- [ ] **Step 5: Extend manifest metadata**

Add optional fields to `InstallManifest`:
- `source_kind`
- `source_repo`
- `release_tag`
- `asset_name`
- `asset_digest`

Keep backward compatibility by making new fields `Option<String>` so existing manifests still deserialize.

- [ ] **Step 6: Run install tests**

Run: `cargo test -p lsp-io-core install`

Expected: existing safety tests still pass.

## Chunk 5: Promote Ada Language Server

### Task 5: Convert ALS registry entry

**Files:**
- Modify: `crates/lsp-io-core/src/server/registry.rs`
- Optional modify: `gui/src/components/Dashboard.ts`

- [ ] **Step 1: Add a failing registry assertion for ALS**

Test:
- `REGISTRY.by_id("ada-language-server").unwrap().install_method.label() == "github release"`
- `can_install` is true on supported current platforms.

- [ ] **Step 2: Add ALS release asset specs**

Use the Mason-backed mapping as the starting point:

```rust
const ALS_RELEASE_ASSETS: &[ReleaseAssetSpec] = &[
    ReleaseAssetSpec {
        target: ReleaseTarget::WindowsX64,
        asset_name: "als-2026.2.202604091-win32-x64.tar.gz",
        archive_format: ArchiveFormat::TarGz,
        binary_path: "integration/vscode/ada/x64/win32/ada_language_server.exe",
        sha256: None,
    },
    // Add darwin_x64, darwin_arm64, linux_x64_gnu, linux_arm64_gnu.
];
```

Use `ReleaseSelector::Tag("2026.2.202604091")` for the first implementation. Fill registry `sha256` values if the GitHub asset API exposes digests consistently during implementation.

- [ ] **Step 3: Replace `manual_entry` with `github_release_entry`**

Keep the existing ALS rationale. Update manual text only for unsupported platforms.

- [ ] **Step 4: Keep Vite mock data aligned**

If the browser preview still uses hardcoded rows, update ALS from `system/manual` to `github release` and `canInstall = true`.

- [ ] **Step 5: Run registry tests**

Run: `cargo test -p lsp-io-core registry`

Expected: ALS is managed and every language still has a registry entry.

## Chunk 6: Documentation And UX Clarity

### Task 6: Document managed archives

**Files:**
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/lsp-io-implementation-plan.md`

- [ ] **Step 1: Update README safety boundaries**

Add that GitHub release archives are managed only when:
- owner/repo is registry allowlisted
- exact asset name is target-matched
- archive format is supported
- download and extracted-size limits pass
- checksum/digest validation passes when available
- extracted binary path stays inside staging root

- [ ] **Step 2: Document optional `GITHUB_TOKEN`**

Mention it is used only for GitHub API rate limits and must not be required for public installs.

- [ ] **Step 3: Update changelog**

Add:
- managed GitHub release archive installer
- Ada Language Server promoted from manual guide to managed install on supported platforms

- [ ] **Step 4: Update implementation plan history**

Mark archive-heavy GitHub release installs as now supported for allowlisted servers, while keeping SDK/toolchain-bound servers manual.

## Chunk 7: Verification

### Task 7: Run full acceptance checks

**Files:**
- No code changes expected.

- [ ] **Step 1: Format**

Run: `cargo fmt`

Expected: no formatting diff remains.

- [ ] **Step 2: Rust tests**

Run: `cargo test`

Expected: all workspace tests pass.

- [ ] **Step 3: Rust check**

Run: `cargo check`

Expected: no errors.

- [ ] **Step 4: GUI build**

Run:

```powershell
cd gui
npm run build
```

Expected: TypeScript and Vite build pass.

- [ ] **Step 5: Dependency audit**

Run:

```powershell
cd gui
npm audit --audit-level=moderate
```

Expected: no moderate-or-higher frontend vulnerabilities.

- [ ] **Step 6: Manual smoke test**

Run:

```powershell
cargo run -p lsp-io-cli -- status
cargo run -p lsp-io-cli -- install ada-language-server
cargo run -p lsp-io-cli -- status
cargo run -p lsp-io-cli -- remove ada-language-server
```

Expected:
- ALS shows `github release`
- install downloads and stages the archive
- status reports a managed install
- remove deletes only the ALS managed cache root

## Deferred Work

- Automatic update checks for GitHub release archive installs.
- Private GitHub repository support.
- Signature verification when upstream publishes detached signatures.
- Bulk promotion of all archive-friendly servers. Promote in small batches after ALS proves the installer path.
