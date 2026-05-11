# SDL-MCP Integration

LSP-IO can export installed language servers into SDL-MCP's `semanticEnrichment.providers.lsp.servers` config shape.

```powershell
cargo run -p lsp-io-cli -- export sdl-mcp F:\Claude\projects\sdl-mcp\sdl-mcp
```

By default, export is conservative:

- installed managed binaries and system `PATH` binaries are exported;
- explicit `.lsp-io.toml` overrides are exported;
- missing/manual commands are skipped;
- `--include-missing` emits preview entries with `readiness: "missing"` and should not be treated as runnable config.

Direct write is opt-in:

```powershell
cargo run -p lsp-io-cli -- export sdl-mcp F:\Claude\projects\sdl-mcp\sdl-mcp --write-config F:\Claude\sdl-mcp\sdlmcp.config.json
```

Use `--enable-semantic-enrichment` only when you also want LSP-IO to set `semanticEnrichment.enabled = true`. Otherwise, the writer upserts LSP server entries and preserves unrelated SDL-MCP config.

## Registry Metadata

Every registry server exposes SDL-MCP launch metadata:

- `serverId`
- default `args`
- SDL-MCP `languages`
- LSP `documentLanguageIds`
- `filePatterns`
- optional `initializationOptions`
- capability hints
- registry readiness status

For example, `typescript-language-server` exports `args = ["--stdio"]`, because the binary must be launched in stdio mode for SDL-MCP's LSP client.

## Overrides

Project-local overrides live in `.lsp-io.toml`:

```toml
[[overrides]]
id = "pyright"
binary_path = "C:/tools/pyright-langserver.cmd"
args = ["--stdio"]
```

Override `binary_path` wins over managed/PATH detection for SDL-MCP export. Override `args` replace registry default args when non-empty.

## Detection Scope

`Install Selected` uses recommendation-grade detection. That scan respects `.gitignore`, skips common generated/temp/tool-cache directories, and removes fixture-only evidence such as `tests/fixtures`, `tests/integration/fixtures`, and `tests/stress/fixtures`.

Use this when you need audit/debug output:

```powershell
cargo run -p lsp-io-cli -- detect F:\Claude\projects\sdl-mcp\sdl-mcp --all-evidence
```

`--all-evidence` can report tracked fixture languages, but ignored temp/worktree/cache content remains excluded.
