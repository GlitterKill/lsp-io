# Changelog

All notable changes to **LSP-IO** are documented in this file.

## [Unreleased]

### Added

- Initial LSP-IO workspace scaffolded from the SCIP-IO Tauri/Rust app shell.
- Language-server registry covering TypeScript, JavaScript, Python, Rust, Go, Java, C#, Ruby, Kotlin, C/C++, Scala, Lua, Swift, and PHP.
- Broad language-server registry expansion across programming, web/framework, config/data/build/infra, shader, hardware, proof, and domain-specific categories.
- Metadata-driven language detection with manifest markers, extension thresholds, directory markers, categories, and confidence labels.
- Managed installs for npm, Go, dotnet tool, gem, and pipx flows.
- Managed GitHub release asset/archive installs with target-specific asset selection, streaming downloads, size limits, digest validation when available, traversal-safe extraction, and app-owned manifests.
- Dashboard category filters for the expanded registry.
- Dashboard server table now shows the full registry instead of only detected languages and supports clickable column sorting.
- CLI polyglot detection regression coverage.
- GitHub release installers for additional guide-only servers, including rust-analyzer, clangd, LuaLS, clojure-lsp, Deno, Expert, nimlangserver, Perl Navigator, v-analyzer, zls, serve-d, Gleam, Millet, Taplo, Docker LS, CUE, KCL, helm-ls, neocmakelsp, mesonlsp, just-lsp, Buf, Postgres LS, GLSL/WGSL/OpenCL servers, Verible, vhdl_ls, Veryl, Marksman, texlab, tinymist, and Regal.
- Status detection that distinguishes app-managed installs from system `PATH` installs.
- GUI dashboard for project language detection, recommended server status, install actions, removal actions, and manual install guidance.
- CLI commands: `detect`, `status`, `install`, `remove`, and `cache-dir`.
- Full-registry SDL-MCP launch metadata and `export sdl-mcp` config fragment/direct-writer support.
- Desktop SDL-MCP config export/write actions after install operations.

### Changed

- Replaced SCIP indexer/run/merge concepts with language server install/status/remove flows.
- Renamed packages, binaries, Tauri product metadata, and GUI text from SCIP-IO to LSP-IO.
- Detection now considers source-file evidence without letting one stray common file dominate the detected language set.
- Ada Language Server is now managed through AdaCore GitHub release archives on supported platforms instead of being guide-only.
- Large GitHub release installs can expose a dashboard warning; clangd is managed through LLVM release archives with a size warning.
- Install Selected now chooses one preferred missing server per selected language instead of installing fallback duplicates for the same language.
- Language detection now respects ignore files and separates recommendation-grade detection from `--all-evidence` audit output.

### Safety

- Removal is limited to app-owned cache directories. System/toolchain installs are never deleted by LSP-IO.
- Managed installs use a staged directory and success manifest before status reports them as app-owned.
- GitHub release asset/archive installs are limited to allowlisted registry entries and exact expected binary paths.
- GitHub release manifest matching now includes version and release-source metadata so pinned release changes are reinstalled instead of silently reporting an older binary as current.
- Tar extraction skips symlink and hardlink entries so large toolchain archives can stage the configured LSP binary without creating archive-controlled links.
- Cache cleanup removes only manifest-backed managed server roots and no longer deletes the configured cache root wholesale.
- Manual/toolchain-hosted server status avoids generic host executable probes that would falsely report an LSP as installed.
