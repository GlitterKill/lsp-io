# LSP-IO

**LSP-IO** is a Rust + Tauri desktop app and CLI for managing language server installs across polyglot projects. It detects project languages from manifests, directory markers, and source-file evidence; recommends the best practical Language Server Protocol implementation for each language; reports status; installs app-owned servers where the install is low-risk; and removes only the binaries it owns.

The GUI intentionally keeps SCIP-IO's dark cyberpunk-corporate layout and controls, but the workflow is now LSP-focused: server recommendations, install state, and cache maintenance instead of SCIP indexing and merge output.

## What It Manages

The registry now covers the original programming-language set plus broad language-server domains:

| Category | Examples | Managed install support |
| --- | --- | --- |
| Programming | TypeScript, JavaScript, Python, Go, C#, Ruby, Bash, Ada/SPARK, F#, Fortran, Elm, plus many toolchain-bound languages | npm, GitHub releases, `go install`, `dotnet tool`, gem, pipx where safe |
| Web/framework | HTML, CSS/Less/Sass, JSON, Angular, Astro, Svelte, Vue, MDX, Tailwind CSS, Emmet, GraphQL | mostly managed npm |
| Config/data/build/infra | YAML, XML, TOML, Docker, Terraform, CUE, Jsonnet, Bicep, Ansible, Helm, CMake, Meson, Just, Make, Nginx, systemd, GitHub Actions, GitLab CI, Protobuf, Thrift, SQL, Postgres SQL, PromQL, OpenAPI | mixed managed npm/go/pipx/GitHub release and system/manual |
| Shader/hardware/proof/domain | GLSL, WGSL, HLSL, QML, OpenCL, SystemVerilog, VHDL, Veryl, DOT, Markdown, LaTeX, Typst, Robot Framework, Gherkin, Rego, Puppet, Lean 4, Coq | managed npm/GitHub releases where assets are clean, otherwise system/manual |

System/manual means LSP-IO detects the server on `PATH` and shows install guidance, but does not remove or overwrite system toolchains. Allowlisted GitHub release assets and archives are managed when the registry defines exact per-platform assets and expected binary paths. Large release installs, such as LLVM/clangd, are still allowed only with an explicit size warning. The full accepted/rejected research table lives in [docs/lsp-io-implementation-plan.md](docs/lsp-io-implementation-plan.md).

The dashboard always shows the full server registry. Detected language chips are still used for the `Install Selected` workflow, and the server table can be sorted by any column header.

## CLI

```powershell
cargo run -p lsp-io-cli -- detect .
cargo run -p lsp-io-cli -- detect . --all-evidence
cargo run -p lsp-io-cli -- status
cargo run -p lsp-io-cli -- install pyright
cargo run -p lsp-io-cli -- export sdl-mcp F:\Claude\projects\sdl-mcp\sdl-mcp --include-missing
cargo run -p lsp-io-cli -- export sdl-mcp F:\Claude\projects\sdl-mcp\sdl-mcp --write-config F:\Claude\sdl-mcp\sdlmcp.config.json
cargo run -p lsp-io-cli -- remove pyright
cargo run -p lsp-io-cli -- cache-dir
```

`detect` is recommendation-grade by default: it respects ignore files and skips common temp, worktree, tool-cache, build-output, and fixture-only evidence so `Install Selected` is not driven by sample content. Use `--all-evidence` when auditing every tracked language signal that is not ignored.

`export sdl-mcp` emits `semanticEnrichment.providers.lsp.servers` config for detected or explicitly overridden servers. By default it only exports entries with an installed command or `.lsp-io.toml` override. `--include-missing` previews missing commands without marking them ready. See [docs/sdl-mcp-integration.md](docs/sdl-mcp-integration.md).

## GUI Development

```powershell
cd gui
npm install
npm run build

cd ..
cargo check
cargo tauri dev
```

The Vite-only browser preview works for layout testing and uses mock data when Tauri commands are unavailable:

```powershell
cd gui
npm run dev -- --host 127.0.0.1
```

## Configuration

Project settings live in `.lsp-io.toml`:

```toml
prefer_path = true
timeout = 300
# cache_dir = "C:/dev/lsp-cache"

[[overrides]]
id = "typescript-language-server"
binary_path = "C:/tools/typescript-language-server.cmd"
args = ["--stdio"]
```

`GITHUB_TOKEN` is optional. When set, LSP-IO uses it only for GitHub release API rate limits while installing public release assets.

## Safety Boundaries

- LSP-IO removes only app-managed cache directories under the LSP-IO cache root.
- Existing `PATH` installs are reported as `system` and are never deleted.
- Managed installs are staged first and treated as installed only after a success manifest and expected binary are present.
- GitHub release assets and archives are managed only for registry-allowlisted repositories with exact target asset names, supported formats, configured download and extracted-size limits, checksum or API digest validation when available, streaming downloads, path-traversal-safe extraction, and portable tar link skipping.
- Clean cache removes only manifest-backed app-managed server roots; it leaves the configured cache directory and unknown child directories in place.
- `cache_dir`, `prefer_path`, and `timeout` are honored by GUI and CLI status/install/remove flows.
- Heavy or toolchain-bound servers stay manual unless the GitHub release asset is a reliable standalone LSP command; large-but-viable assets show an install warning.

## Sources Used For The Registry

- [Language Server Protocol overview](https://learn.microsoft.com/en-us/visualstudio/extensibility/language-server-protocol?view=visualstudio)
- [Microsoft Language Servers implementor list](https://microsoft.github.io/language-server-protocol/implementors/servers/)
- [Mason registry package list](https://mason-registry.dev/registry/list)
- [nvim-lspconfig server configs](https://github.com/neovim/nvim-lspconfig)
- [typescript-language-server](https://github.com/typescript-language-server/typescript-language-server)
- [Pyright](https://github.com/microsoft/pyright)
- [rust-analyzer](https://rust-analyzer.github.io/)
- [gopls](https://pkg.go.dev/golang.org/x/tools/gopls)
- [Eclipse JDT LS](https://github.com/eclipse-jdtls/eclipse.jdt.ls)
- [clangd](https://clangd.llvm.org/)
- [csharp-ls](https://github.com/razzmatazz/csharp-language-server)
- [Ruby LSP](https://shopify.github.io/ruby-lsp/)
- [Kotlin LSP](https://github.com/Kotlin/kotlin-lsp)
- [LuaLS](https://luals.github.io/)
- [SourceKit-LSP](https://github.com/swiftlang/sourcekit-lsp)
- [Metals](https://scalameta.org/metals/)
