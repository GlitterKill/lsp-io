# LSP-IO Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development if parallel workers are available, or execute this plan directly with verification after each chunk. Steps use Markdown checkbox syntax for tracking.

**Goal:** Build LSP-IO as a Tauri desktop app cloned from SCIP-IO's layout, dark theme, and operational controls, but focused on language server discovery, managed installation, status checks, and removal.

**Architecture:** Start from SCIP-IO's Rust workspace and Vite TypeScript GUI so the app keeps the proven shell, titlebar, panels, table styling, logging, and progress model. Replace the SCIP indexing pipeline with a language-server registry, installation manager, and LSP-specific UI copy. Keep high-footprint/toolchain-bound servers visible even when they are not safe to auto-install, because hiding them would make the recommendations less honest.

**Tech Stack:** Rust 2024 workspace, Tauri 2, Vite 6, TypeScript, platform package managers (`npm`, `go`, `dotnet`, `gem`, `pipx`), allowlisted GitHub release assets/archives, and explicit system/manual guidance for toolchain-bound servers.

---

## Research Decisions

The selection rule is "most robust language intelligence for the smallest practical install footprint." That favors compiler-team or compiler-adjacent servers where available, then widely adopted community servers with standard LSP transport.

| Language | Recommended LSP | Install support in v1 | Rationale |
| --- | --- | --- | --- |
| TypeScript / JavaScript | `typescript-language-server` + `typescript` | Managed npm install | Thin standard-LSP layer over TypeScript `tsserver`; better semantic fidelity than lighter lint-only servers. |
| Python | `pyright-langserver` from `pyright` | Managed npm install | Full-featured, standards-based Microsoft type checker; larger than Ruff, but Ruff is not a full semantic LSP replacement. |
| Rust | `rust-analyzer` | Managed GitHub release | Official/de facto Rust LSP; upstream publishes exact standalone release assets, while rustup remains a valid system/manual alternative. |
| Go | `gopls` | Managed `go install` when Go is present | Official Go Team LSP with broad IDE features; requires Go toolchain but installs a single server binary. |
| Java | Eclipse JDT LS | System/manual in v1 | Most complete Java LSP by capability; footprint is high and Java 21 is required, so automatic app-managed install is deferred. |
| C# | `csharp-ls` | Managed `dotnet tool` install | Roslyn-based LSP with a much smaller footprint than OmniSharp/C# Dev Kit. |
| Ruby | Shopify `ruby-lsp` | Managed gem install | Modern Ruby LSP with broad editor features and add-on support. |
| Kotlin | JetBrains `kotlin-lsp` | System/manual in v1 | Official Kotlin LSP exists but is still pre-alpha and bundled as platform archives; expose as recommended but do not overstate maturity. |
| C / C++ | `clangd` | Managed GitHub release with size warning | LLVM/Clang-backed standard choice; best capability, but install pulls a large LLVM distribution and must be surfaced honestly. |
| Scala | Metals | System/manual in v1 | Scala Center/Scalameta server with rich features; managed install needs Coursier/JVM handling and is heavier than v1 should hide. |
| Lua | LuaLS `lua-language-server` | Managed GitHub release | Rich Lua-specific diagnostics, annotations, formatting, and completion; release layout is modeled by exact per-platform assets. |
| Swift | `sourcekit-lsp` | System/manual in v1 | Official Swift LSP bundled with Swift toolchains/Xcode; app should detect rather than duplicate the toolchain. |
| PHP | `phpactor` | System/manual in v1 | Best open-source PHP LSP fit when balancing capability and availability; Composer-managed install can follow later. |

Primary evidence checked:

- Microsoft describes LSP as a JSON-RPC protocol where a language server runs in its own process and can be consumed by multiple IDEs.
- `typescript-language-server` documents itself as a thin LSP interface over TypeScript/VS Code language-feature code and installs via npm.
- Pyright describes itself as a full-featured, standards-based Python type checker.
- `rust-analyzer` documents itself as an LSP implementation for Rust with prebuilt binaries.
- `gopls` is documented by the Go project as the official Go language server.
- Eclipse JDT LS documents Maven/Gradle support, Java 1.8 through 25 project support, and a Java 21 runtime requirement.
- `clangd` is LLVM/Clang based and provides completion, diagnostics, and go-to-definition.
- `csharp-ls` is Roslyn-based and installs as a `.NET` tool.
- Shopify Ruby LSP implements LSP for Ruby and lists broad editor features.
- JetBrains `kotlin-lsp` is official but pre-alpha.
- LuaLS and SourceKit-LSP document LSP support and broad language intelligence.
- Metals documents rich Scala IDE features and build tool support.

## Chunk 1: Scaffold from SCIP-IO

**Files:**

- Copy from `F:\Claude\projects\scip-io`: workspace metadata, `gui`, `src-tauri`, `crates`, scripts, docs assets, license.
- Delete generated/heavy artifacts: `.git`, `target`, `tmp-release-test`, `*.scip`.
- Rename package names from `scip-io` to `lsp-io`.

- [x] Copy the source app into the blank folder.
- [x] Remove generated artifacts and stale SCIP sample indexes.
- [x] Rename Cargo packages, binary names, Tauri product metadata, GUI package name, and title strings.
- [x] Run `rg "SCIP|scip|index.scip|Indexer"` and triage every hit.

## Chunk 2: Replace Core Domain

**Files:**

- Create/replace `crates/lsp-io-core/src/language.rs`.
- Create/replace `crates/lsp-io-core/src/server/registry.rs`.
- Create/replace `crates/lsp-io-core/src/server/install.rs`.
- Create/replace `crates/lsp-io-core/src/server/mod.rs`.
- Create/replace `crates/lsp-io-core/src/config/mod.rs`.
- Create/replace `crates/lsp-io-core/src/progress.rs`.
- Modify `crates/lsp-io-core/src/lib.rs`.

- [x] Define language detection for the supported matrix.
- [x] Define `ServerEntry`, install method metadata, footprint class, maturity, source URL, and rationale.
- [x] Implement cache path resolution from project settings with user-cache defaults.
- [x] Implement status resolution using the configured `prefer_path` ordering.
- [x] Implement managed installs for npm, Go, dotnet tool, and gem. Initial Rust manual guidance is superseded by the allowlisted GitHub release promotion in Chunk 8.
- [x] Implement managed removal only for app-owned files and return a clear message for system/manual servers.
- [x] Add focused Rust tests for registry coverage, detection manifests, shared install roots, manifest-backed installs, and non-destructive removal behavior.

## Chunk 3: Replace Tauri Commands

**Files:**

- Replace `src-tauri/src/commands.rs`.
- Modify `src-tauri/src/lib.rs`.
- Modify `src-tauri/Cargo.toml`.

- [x] Expose `detect_languages`, `get_server_status`, `install_servers`, `install_one_server`, `remove_one_server`, `clean_cache`, `get_config`, `save_config`, and `check_updates`.
- [x] Emit progress events using the existing frontend event shape.
- [x] Keep command argument names camelCase-compatible with Tauri `invoke`.
- [x] Ensure system/manual servers never get deleted by app commands.

## Chunk 4: Adapt GUI

**Files:**

- Modify `gui/src/state/store.ts`.
- Modify `gui/src/bridge/tauri.ts`.
- Replace copy in `Dashboard.ts`, `IndexProgress.ts`, `Results.ts`, `Settings.ts`, `Titlebar.ts`, `StatusBadge.ts`.
- Keep existing CSS files unless text/container fit needs small changes.
- Modify `gui/index.html`.

- [x] Rename the brand to LSP-IO and replace SCIP indexing copy.
- [x] Replace "Indexer Status" with "Language Server Status".
- [x] Add install/remove/status action buttons per server row.
- [x] Add "Install Selected" flow from detected languages.
- [x] Keep mock data for browser/Vite development only, while preserving real Tauri errors.
- [x] Ensure table text fits at mobile/desktop widths without layout shifts.

## Chunk 5: CLI and Docs

**Files:**

- Replace `crates/lsp-io-cli/src/main.rs` and CLI modules.
- Replace `README.md`, `CHANGELOG.md`, and relevant release/install script text.
- Add or update docs assets to avoid SCIP-specific wording.

- [x] Provide `lsp-io detect`, `lsp-io status`, `lsp-io install`, `lsp-io remove`, and `lsp-io cache-dir`.
- [x] Document the LSP candidate matrix and why some servers are system/manual in v1.
- [x] Document prerequisites for managed installers.
- [x] Keep release docs consistent with renamed binaries.

## Chunk 6: Verification

- [x] Run `cargo fmt`.
- [x] Run `cargo test`.
- [x] Run `npm install` in `gui` if needed.
- [x] Run `npm run build` in `gui`.
- [x] Run `cargo check`.
- [x] Start `npm run dev -- --host 127.0.0.1` in `gui`.
- [x] Verify the local GUI in a browser at desktop and mobile-ish widths.

## Chunk 7: Broad LSP Registry Expansion

**Goal:** Expand LSP-IO beyond the original v1 programming-language set into every currently viable cross-platform standalone LSP class found in the Microsoft LSP implementor list, Mason registry, nvim-lspconfig, and upstream project documentation. Broad scope includes programming languages, web/framework files, config/data/query/build/infra formats, shader languages, hardware-description languages, proof assistants, and maintained domain-specific languages.

**Viability rule:** Accept a server only when it is cross-platform, callable as a standalone LSP or stable toolchain command, not archived/proprietary-only, and has a credible install or manual setup path. Prefer official/compiler-team servers first, then widely adopted maintained community servers with smaller footprints.

**Files:**

- Extend `crates/lsp-io-core/src/language.rs`.
- Extend `crates/lsp-io-core/src/server/registry.rs`.
- Extend `crates/lsp-io-core/src/server/install.rs`.
- Extend `crates/lsp-io-core/src/server/mod.rs`.
- Extend `src-tauri/src/commands.rs`.
- Extend `gui/src/state/store.ts`, `gui/src/bridge/tauri.ts`, `gui/src/components/Dashboard.ts`, and component CSS.
- Update `README.md` and `CHANGELOG.md`.

- [x] Add `LanguageCategory`: `Programming`, `Web`, `Config`, `Data`, `Build`, `Infra`, `Shader`, `Hardware`, `Proof`, `Framework`, `DomainSpecific`.
- [x] Replace manifest-only detection with metadata-owned manifest markers, extension markers, optional directory markers, and per-language extension thresholds.
- [x] Add `DetectionConfidence` so the UI/CLI can distinguish high-signal manifests from lower-signal extension-only matches.
- [x] Add broad registry entries in tiered batches: high-value web/config servers, toolchain-bound mainstream servers, then specialized shader/hardware/proof/domain servers.
- [x] Add conservative `pipx` managed installs for small Python-packaged servers.
- [x] Keep managed installs limited to predictable package-manager-backed binaries and allowlisted GitHub release assets/archives; keep SDK/toolchain-bound servers as system/manual.
- [x] Add category filters and a concise manual/system explanation to the dashboard without changing the existing layout/theme.
- [x] Add registry coverage tests proving every `LanguageKind` has at least one server entry.
- [x] Add fixture-style detection tests for programming, web, config/data, infra, shader, hardware, and proof categories.
- [x] Add duplicate-install-root tests for shared TypeScript/JavaScript and HTML/CSS/JSON servers.
- [x] Add a CLI `detect` integration test against a synthetic polyglot project.

### Broad Registry Research Artifact

Primary sources used as the baseline:

- [Microsoft Language Servers implementor list](https://microsoft.github.io/language-server-protocol/implementors/servers/)
- [Mason registry package list](https://mason-registry.dev/registry/list)
- [nvim-lspconfig server config registry](https://github.com/neovim/nvim-lspconfig)
- Upstream project documentation linked per accepted candidate below

Accepted candidates:

| Area | Language/domain | Recommended server | Source | LSP-IO install | Maturity | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Programming | Ada/SPARK | Ada Language Server | [AdaCore ALS](https://github.com/AdaCore/ada_language_server) | Managed GitHub release | Stable | Official AdaCore server; platform release archive layout is now modeled by the registry. |
| Programming | Bash | bash-language-server | [bash-lsp](https://github.com/bash-lsp/bash-language-server) | Managed npm | Stable | Small npm package and predictable `bash-language-server` binary. |
| Programming | Clojure | clojure-lsp | [clojure-lsp](https://github.com/clojure-lsp/clojure-lsp) | Managed GitHub release | Stable | Strong project-aware Clojure analysis; upstream publishes platform native archives. |
| Programming | Dart | Dart SDK LSP | [Dart SDK](https://github.com/dart-lang/sdk) | System/manual | Stable | Official, SDK-bound. |
| Programming | Deno | `deno lsp` | [Deno](https://github.com/denoland/deno) | Managed GitHub release | Stable | Official server is bundled with the Deno standalone release asset. |
| Programming | Elixir | Expert, ElixirLS fallback | [Expert](https://github.com/elixir-lang/expert) | Managed GitHub release | Preview | Official direction is Expert; fallback remains ElixirLS where Expert is not ready. |
| Programming | Erlang | ELP | [Erlang Language Platform](https://github.com/WhatsApp/erlang-language-platform) | System/manual | Stable | More active modern option than older Erlang LS variants. |
| Programming | F# | FsAutoComplete | [FsAutoComplete](https://github.com/ionide/FsAutoComplete) | Managed dotnet tool | Stable | Established Ionide server and predictable .NET tool install. |
| Programming | Fortran | fortls | [fortls](https://github.com/fortran-lang/fortls) | Managed pipx | Stable | Fortran-lang maintained and compact. |
| Programming | Haskell | Haskell Language Server | [HLS](https://github.com/haskell/haskell-language-server) | System/manual | Stable | Toolchain-bound through GHC/GHCup. |
| Programming | Julia | LanguageServer.jl | [LanguageServer.jl](https://github.com/julia-vscode/LanguageServer.jl) | System/manual | Stable | Runs through Julia package/toolchain. |
| Programming | Nim | nimlangserver | [Nim langserver](https://github.com/nim-lang/langserver) | Managed GitHub release | Stable | Nim organization language server with clean native release archives. |
| Programming | OCaml/Reason | ocamllsp | [OCaml LSP](https://github.com/ocaml/ocaml-lsp) | System/manual | Stable | Official OCaml Platform server. |
| Programming | Perl | Perl Navigator | [Perl Navigator](https://github.com/bscan/PerlNavigator) | Managed GitHub release | Stable | More actively maintained than older Perl LS choices; upstream publishes x64 archives. |
| Programming | PowerShell | PowerShell Editor Services | [PowerShell Editor Services](https://github.com/PowerShell/PowerShellEditorServices) | System/manual | Stable | Microsoft-maintained and tied to PowerShell tooling. |
| Programming | R | R `languageserver` | [R languageserver](https://github.com/REditorSupport/languageserver) | System/manual | Stable | R package/toolchain install is the durable path. |
| Programming | Racket | racket-langserver | [racket-langserver](https://github.com/jeapostrophe/racket-langserver) | System/manual | Stable | Dedicated Racket LSP. |
| Programming | Raku | Raku Navigator | [Raku Navigator](https://github.com/bscan/RakuNavigator) | System/manual | Stable | Maintained by Perl Navigator author. |
| Programming | ReScript | ReScript language server | [rescript-vscode](https://github.com/rescript-lang/rescript-vscode) | System/manual | Stable | Official language tooling. |
| Programming | V | v-analyzer | [v-analyzer](https://github.com/vlang/v-analyzer) | Managed GitHub release | Stable | Preferred over older vls; release archives have predictable binary paths. |
| Programming | Vala | vala-language-server | [Vala LS](https://github.com/vala-lang/vala-language-server) | System/manual | Stable | Maintained under Vala org. |
| Programming | Zig | zls | [zls](https://github.com/zigtools/zls) | Managed GitHub release | Stable | Canonical Zig LSP with zip/tar.xz platform archives. |
| Programming | Nix | nil | [nil](https://github.com/oxalica/nil) | System/manual | Stable | Lower footprint than nixd for default recommendation. |
| Programming | Ballerina | Ballerina Language Server | [ballerina-lang](https://github.com/ballerina-platform/ballerina-lang) | System/manual | Stable | Official and SDK-bound. |
| Programming | Chapel | Chapel Language Server | [Chapel](https://github.com/chapel-lang/chapel) | System/manual | Stable | Official ecosystem server. |
| Programming | Crystal | Crystalline | [Crystalline](https://github.com/elbywan/crystalline) | System/manual | Stable | More capable maintained option than older Scry default. |
| Programming | D | serve-d | [serve-d](https://github.com/Pure-D/serve-d) | Managed GitHub release | Stable | Widely used DUB-aware server with platform archives. |
| Programming | Elm | elm-language-server | [Elm LS](https://github.com/elm-tooling/elm-language-server) | Managed npm | Stable | Clean npm package and predictable binary. |
| Programming | Gleam | Gleam language server | [Gleam](https://github.com/gleam-lang/gleam) | Managed GitHub release | Stable | Official server is part of the standalone Gleam release binary. |
| Programming | Groovy | Groovy Language Server | [Groovy LS](https://github.com/GroovyLanguageServer/groovy-language-server) | System/manual | Stable | JVM packaging is better left explicit. |
| Programming | Haxe | Haxe Language Server | [Haxe LS](https://github.com/vshaxe/haxe-language-server) | System/manual | Stable | Haxe Foundation/vshaxe path. |
| Programming | Idris2 | idris2-lsp | [idris2-lsp](https://github.com/idris-community/idris2-lsp) | System/manual | Stable | Toolchain-bound proof/programming language server. |
| Proof | Lean 4 | Lean language server | [Lean 4](https://github.com/leanprover/lean4) | System/manual | Stable | Official elan/lake toolchain path. |
| Proof | Coq | coq-lsp | [coq-lsp](https://github.com/ejgallego/coq-lsp) | System/manual | Stable | Modern Coq LSP. |
| Programming | Common Lisp | cl-lsp | [cl-lsp](https://github.com/cxxxr/cl-lsp) | System/manual | Stable | Standalone Common Lisp LSP. |
| Programming | Standard ML | Millet | [Millet](https://github.com/azdavis/millet) | Managed GitHub release | Stable | Maintained SML LSP with gzip-compressed release binaries. |
| Web | HTML | vscode-html-language-server | [vscode-langservers-extracted](https://github.com/hrsh7th/vscode-langservers-extracted) | Managed npm | Stable | Shares one npm install with CSS and JSON. |
| Web | CSS/Less/Sass | vscode-css-language-server | [vscode-langservers-extracted](https://github.com/hrsh7th/vscode-langservers-extracted) | Managed npm | Stable | Shares one npm install with HTML and JSON. |
| Data | JSON | vscode-json-language-server | [vscode-langservers-extracted](https://github.com/hrsh7th/vscode-langservers-extracted) | Managed npm | Stable | Schema-aware JSON support. |
| Framework | Angular | Angular Language Server | [Angular language service](https://angular.dev/tools/language-service) | Managed npm | Stable | Official Angular server. |
| Framework | Astro | Astro Language Server | [Astro language tools](https://github.com/withastro/language-tools) | Managed npm | Stable | Official Astro package. |
| Framework | Svelte | Svelte Language Server | [Svelte language tools](https://github.com/sveltejs/language-tools) | Managed npm | Stable | Official Svelte package. |
| Framework | Vue | Vue Language Server | [Vue language tools](https://github.com/vuejs/language-tools) | Managed npm | Stable | Official successor to Vetur. |
| Web | MDX | MDX Language Server | [MDX analyzer](https://github.com/mdx-js/mdx-analyzer) | Managed npm | Stable | Official MDX project tooling. |
| Framework | Tailwind CSS | Tailwind CSS Language Server | [Tailwind IntelliSense](https://github.com/tailwindlabs/tailwindcss-intellisense) | Managed npm | Stable | Official Tailwind Labs server. |
| Web | Emmet | emmet-language-server | [emmet-language-server](https://github.com/olrtg/emmet-language-server) | Managed npm | Stable | Small standalone Emmet LSP. |
| Data | GraphQL | GraphQL Language Service CLI | [GraphQL language service](https://github.com/graphql/graphiql/tree/main/packages/graphql-language-service-cli) | Managed npm | Stable | Official GraphQL Foundation tooling. |
| Data | YAML | yaml-language-server | [YAML LS](https://github.com/redhat-developer/yaml-language-server) | Managed npm | Stable | Schema-aware YAML support. |
| Data | XML | LemMinX | [LemMinX](https://github.com/eclipse-lemminx/lemminx) | System/manual | Stable | Mature Java/XML server; keep install explicit. |
| Data | TOML | Taplo | [Taplo](https://github.com/tamasfe/taplo) | Managed GitHub release | Stable | Compact Rust TOML server with zip/gzip release assets. |
| Infra | Docker/Compose/Bake | Docker Language Server | [Docker LS](https://github.com/docker/docker-language-server) | Managed GitHub release | Stable | Official Docker server publishes standalone release binaries. |
| Infra | Dockerfile fallback | dockerfile-language-server | [dockerfile-language-server-nodejs](https://github.com/rcjsuen/dockerfile-language-server-nodejs) | Managed npm | Stable | Low-risk Dockerfile-only fallback. |
| Infra | Terraform | terraform-ls | [terraform-ls](https://github.com/hashicorp/terraform-ls) | System/manual | Stable | Official HashiCorp server. |
| Config | CUE | CUE language server | [CUE](https://github.com/cue-lang/cue) | Managed GitHub release | Stable | Official CUE binary provides the language-server path and clean release archives. |
| Config | Jsonnet | jsonnet-language-server | [Jsonnet LS](https://github.com/grafana/jsonnet-language-server) | Managed go install | Stable | Grafana Go module install path. |
| Config | KCL | KCL Language Server | [KCL](https://github.com/kcl-lang/kcl) | Managed GitHub release | Stable | Official KCL release archives include `kcl-language-server`. |
| Infra | Bicep | Bicep Language Server | [Bicep](https://github.com/Azure/bicep) | System/manual | Stable | Microsoft toolchain-bound server. |
| Infra | Ansible | ansible-language-server | [vscode-ansible](https://github.com/ansible/vscode-ansible) | System/manual | Stable | Often requires Python/Ansible project environment. |
| Infra | Helm | helm-ls | [helm-ls](https://github.com/mrjosh/helm-ls) | Managed GitHub release | Stable | Go-based Helm chart server with standalone release assets. |
| Build | CMake | neocmakelsp | [neocmakelsp](https://github.com/Decodetalkers/neocmakelsp) | Managed GitHub release | Stable | Smaller Rust option over older Python server; release archives are predictable. |
| Build | Meson | mesonlsp | [mesonlsp](https://github.com/JCWasmx86/mesonlsp) | Managed GitHub release | Stable | Dedicated Meson server with platform archives. |
| Build | Just | just-lsp | [just-lsp](https://github.com/terror/just-lsp) | Managed GitHub release | Stable | Small Justfile server with platform release archives. |
| Build | Make | make-lsp | [make-lsp-vscode](https://github.com/alexclewontin/make-lsp-vscode) | System/manual | Preview | Young but viable Makefile-specific LSP. |
| Infra | Nginx | nginx-language-server | [nginx-language-server](https://github.com/pappasam/nginx-language-server) | Managed pipx | Stable | Small Python package. |
| Infra | systemd | systemd-language-server | [systemd-language-server](https://github.com/psacawa/systemd-language-server) | Managed pipx | Stable | Small Python package. |
| Infra | GitHub Actions | GitHub Actions Language Server | [actions/languageservices](https://github.com/actions/languageservices) | System/manual | Preview | Official service exists; standalone executable remains wrapper-based. |
| Infra | GitLab CI | gitlab-ci-ls | [gitlab-ci-ls](https://github.com/alesbrelih/gitlab-ci-ls) | System/manual | Stable | Cross-platform packages exist, but no simple universal npm/go path. |
| Data | Protobuf | Buf Language Server | [Buf](https://github.com/bufbuild/buf) | Managed GitHub release | Stable | Modern Buf-managed Protobuf tooling with platform archives. |
| Data | Thrift | thrift-ls | [thrift-ls](https://github.com/joyme123/thrift-ls) | Managed go install | Preview | Straightforward Go install, smaller ecosystem. |
| Data | SQL | sqls | [sqls](https://github.com/sqls-server/sqls) | Managed go install | Stable | General SQL LSP with one-binary install. |
| Data | Postgres SQL | Postgres Language Server | [Postgres LS](https://github.com/supabase-community/postgres-language-server) | Managed GitHub release | Stable | Postgres-specific parser fidelity; release assets include standalone `postgrestools` binaries. |
| Domain-specific | PromQL | promql-langserver | [promql-langserver](https://github.com/prometheus-community/promql-langserver) | Managed go install | Preview | Compact Go LSP, but older/smaller project. |
| Data | OpenAPI | yaml-language-server schemas | [YAML LS](https://github.com/redhat-developer/yaml-language-server) | Managed npm | Stable | Prefer schema-aware YAML/JSON handling over weaker OpenAPI-only servers. |
| Shader | GLSL | glsl_analyzer | [glsl_analyzer](https://github.com/nolanderc/glsl_analyzer) | Managed GitHub release | Stable | Smaller Rust option than older GLSL LS; platform zip assets are clean. |
| Shader | WGSL | wgsl-analyzer | [wgsl-analyzer](https://github.com/wgsl-analyzer/wgsl-analyzer) | Managed GitHub release | Stable | Dedicated WebGPU shader server with zip/gzip assets. |
| Shader | HLSL | HLSL Tools | [HLSL Tools](https://github.com/tgjones/HlslTools) | System/manual | Stable | Most mature open HLSL tooling found. |
| Framework | QML | qmlls | [Qt QML LS](https://github.com/qt/qtdeclarative) | System/manual | Stable | Official Qt toolchain server. |
| Shader | OpenCL | opencl-language-server | [OpenCL LS](https://github.com/Galarius/opencl-language-server) | Managed GitHub release | Stable | Dedicated OpenCL server with small platform archives. |
| Hardware | SystemVerilog | Verible Verilog LS | [Verible](https://github.com/chipsalliance/verible) | Managed GitHub release | Stable | CHIPS Alliance maintained; selected over less mature alternatives. |
| Hardware | VHDL | vhdl_ls | [vhdl_ls](https://github.com/VHDL-LS/rust_hdl) | Managed GitHub release | Stable | Compact Rust server with platform archives where available. |
| Hardware | Veryl | Veryl Language Server | [Veryl](https://github.com/veryl-lang/veryl) | Managed GitHub release | Stable | Official Veryl server with platform archives. |
| Domain-specific | DOT | dot-language-server | [dot-language-server](https://github.com/nikeee/dot-language-server) | Managed npm | Stable | Small npm package for Graphviz DOT. |
| Data | Markdown | Marksman | [Marksman](https://github.com/artempyanykh/marksman) | Managed GitHub release | Stable | Markdown LSP with standalone release binaries; Mermaid-specific LSP remains deferred. |
| Domain-specific | LaTeX | texlab | [texlab](https://github.com/latex-lsp/texlab) | Managed GitHub release | Stable | Mature LaTeX LSP with platform archives. |
| Domain-specific | Typst | tinymist | [tinymist](https://github.com/Myriad-Dreamin/tinymist) | Managed GitHub release | Stable | Supersedes older typst-lsp default; platform archives are allowlisted. |
| Domain-specific | Robot Framework | RobotCode | [RobotCode](https://github.com/robotcodedev/robotcode) | System/manual | Stable | Active Robot Framework tooling. |
| Domain-specific | Gherkin/Cucumber | Cucumber Language Server | [Cucumber LS](https://github.com/cucumber/language-server) | Managed npm | Stable | Official Cucumber server. |
| Domain-specific | Rego | Regal | [Regal](https://github.com/StyraInc/regal) | Managed GitHub release | Stable | Maintained OPA/Rego language server path with standalone release binaries. |
| Infra | Puppet | Puppet Editor Services | [Puppet Editor Services](https://github.com/lingua-pupuli/puppet-editor-services) | System/manual | Stable | Maintained Puppet community server. |

## Chunk 8: GitHub Release/Archive Promotion Audit

**Goal:** Review every guide/system install row and promote only servers that are installable by LSP-IO's reusable GitHub release asset/archive installer. The audit is intentionally limited to GitHub release assets and archives; package-manager-only, SDK-bound, editor-extension-only, or runtime-script installs stay guide-only.

**Implementation notes:**

- [x] Extend the reusable installer for `.tar.xz`, gzip-compressed single binaries, and raw GitHub release binaries.
- [x] Stream downloads to the staging directory instead of buffering release assets in memory, so large allowlisted assets can be handled without multi-GB RAM spikes.
- [x] Keep download-size and extracted-size limits per server.
- [x] Add `install_warning` metadata and surface it in the dashboard for large-but-accepted installs.
- [x] Promote only exact owner/repo/tag/asset-name/binary-path mappings.

Promoted from Guide to Install:

| Server IDs | GitHub release fit | Warning |
| --- | --- | --- |
| `rust-analyzer`, `lua-language-server`, `clojure-lsp`, `deno-lsp`, `expert`, `nimlangserver`, `perl-navigator`, `v-analyzer`, `zls`, `serve-d`, `gleam-lsp`, `millet` | Standalone release binaries or clean platform archives with predictable binary paths. | None. |
| `taplo`, `docker-language-server`, `cue-lsp`, `kcl-language-server`, `helm-ls`, `neocmakelsp`, `mesonlsp`, `just-lsp`, `buf-language-server`, `postgres-language-server` | Standalone CLI/LSP release assets; selected assets are exact per platform. | None. |
| `glsl-analyzer`, `wgsl-analyzer`, `opencl-language-server`, `verible`, `vhdl-ls`, `veryl-ls`, `marksman`, `texlab`, `tinymist`, `regal` | Dedicated language-server release assets or official tool binaries that expose the server command. | None. |
| `clangd` | LLVM publishes exact clangd-containing release archives. Size was the blocker, so it is included per policy. | Large LLVM archive warning. |

Reviewed and kept guide-only:

| Server IDs | Reason not promoted through the GitHub release/archive installer |
| --- | --- |
| `jdtls`, `kotlin-lsp`, `metals`, `lemminx`, `bicep-language-server`, `powershell-editor-services`, `puppet-editor-services` | Release packaging is JVM/.NET/script/extension oriented or lacks a stable standalone executable path for this installer. |
| `sourcekit-lsp`, `dart-sdk-lsp`, `julia-language-server`, `r-languageserver`, `racket-langserver`, `ballerina-language-server`, `chapel-language-server`, `qmlls` | SDK or language-toolchain managed; the GitHub release asset is absent or not a standalone LSP server install. |
| `elp`, `haskell-language-server`, `lean-language-server`, `coq-lsp`, `idris2-lsp` | Version/toolchain compatibility is not only a size issue; selecting one release asset would be misleading for project-specific compiler/runtime versions. |
| `phpactor`, `ocamllsp`, `raku-navigator`, `rescript-language-server`, `vala-language-server`, `nil`, `groovy-language-server`, `haxe-language-server`, `cl-lsp`, `make-lsp`, `github-actions-language-server`, `hlsl-tools`, `robotcode` | No suitable current GitHub release asset for this installer, or the asset is source/package/editor-extension shaped rather than a standalone executable. |
| `crystalline`, `gitlab-ci-ls` | Current GitHub release assets are not viable cross-platform default installs for the registry. |
| `terraform-ls` | Current GitHub release metadata exposes no downloadable assets for this installer; upstream distribution remains external. |

Rejected or deferred candidates:

| Candidate/category | Source example | Decision | Reason |
| --- | --- | --- | --- |
| Proprietary-only servers | DelphiLSP, Sigasi VHDL, IBM/Broadcom mainframe language extensions | Rejected | Violates open cross-platform standalone requirement or creates vendor lock-in. |
| Archived/unmaintained servers | Microsoft Python Language Server, Palantir `python-language-server`, Flow language server, old `terraform-lsp` | Rejected | Superseded by maintained alternatives already selected. |
| Editor-extension-only integrations | VS Code C++ extension, Salesforce bundled extension servers, several vendor IDE extensions | Rejected | No stable standalone CLI entrypoint suitable for LSP-IO management. |
| OpenAPI-only AML server | AML Language Server | Deferred | YAML/JSON schema-aware support is more robust and lower risk for OpenAPI projects. |
| Mermaid-specific tooling | Mermaid editor integrations | Deferred | No sufficiently mature standalone LSP; Markdown support is covered by Marksman. |
| Tiny niche/game DSLs | DreamMaker, Papyrus, DenizenScript, Minecraft Data Pack, KerboScript, Lox, Tibbo Basic | Deferred | Too narrow for the default registry unless both Mason and nvim-lspconfig expose a maintained standalone package. |
| Legacy/superseded framework servers | Vetur, `typst-lsp`, Scry | Rejected | Modern successors are selected: Vue language-tools, tinymist, and Crystalline. |

## Known v1 Tradeoffs

- Many first-class recommendations are intentionally system/manual when the server is tied to a large SDK, project-specific language toolchain, JVM/.NET runtime, script launcher, or editor-extension package. Platform-specific release assets are managed only after the registry defines exact assets, size limits, warnings where needed, and expected binary paths.
- The app does not launch or supervise LSP protocol sessions in v1. It manages binaries and status; actual editor/client integration can be a follow-up.
- Update checking can be coarse in v1. Exact package-manager-specific latest-version probing is useful but not required for a reliable install/status/remove loop.
