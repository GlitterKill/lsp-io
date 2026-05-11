# Full-Registry SDL-MCP LSP Readiness Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents are explicitly requested) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

## Summary

Make every current LSP-IO registry server export SDL-MCP-ready launch metadata, and make SDL-MCP consume those entries through a generic LSP-native enrichment path instead of the previous TypeScript-only path.

This is a cross-repo contract:

- LSP-IO owns installer/status/detection/export and must never write runnable SDL-MCP config for a missing or unresolved command unless the user explicitly requests preview output.
- SDL-MCP owns semantic-enrichment execution and must be able to launch any configured LSP server, initialize it, run supported capabilities, and record meaningful provider results.

Current verified constraint:

- LSP-IO has 103 server entries across 102 language/domain kinds.
- SDL-MCP currently uses configured stdio LSP servers but rejects non-`typescript` LSP enrichment in its source-selection path.

## Required Outcomes

- Every LSP-IO `ServerEntry` has SDL-MCP launch metadata through the registry contract:
  `serverId`, `args`, SDL-MCP `languages`, document language IDs, file patterns, initialization options, capability expectations, and readiness status.
- LSP-IO export/write behavior stays conservative:
  export resolved installed commands or explicit overrides by default; only include missing/manual tools in preview mode; never mark missing tools as ready.
- LSP-IO detection is recommendation-grade by default:
  respect `.gitignore`, skip generated/temp/tool-cache directories, and keep fixture-only evidence out of install recommendations while still supporting all-evidence audits.
- SDL-MCP replaces the TypeScript-only guard with a capability-driven LSP runner:
  initialize configured servers, open matching documents, request document symbols and diagnostics when supported, and run definition/reference/call-related capabilities when available.
- Existing SDL-MCP tree-sitter-assisted call-definition enrichment remains the higher-precision mode for current adapter languages, but generic LSP-native ingestion works for configured languages without SDL-MCP tree-sitter adapters.

## Public Interfaces

LSP-IO registry metadata is required for every server:

- `serverId`
- `args`
- `languages`
- `documentLanguageIds`
- `filePatterns`
- `initializationOptions`
- `capabilities`
- `readiness`

LSP-IO CLI:

```powershell
lsp-io export sdl-mcp <repo> [--write-config <path>] [--include-missing] [--validate-launch]
```

SDL-MCP config remains backward-compatible:

```json
{
  "semanticEnrichment": {
    "providers": {
      "lsp": {
        "servers": {
          "typescript-language-server": {
            "serverId": "typescript-language-server",
            "command": "typescript-language-server",
            "args": ["--stdio"],
            "languages": ["typescript"],
            "initializationOptions": {}
          }
        }
      }
    }
  }
}
```

SDL-MCP may accept optional per-server hints such as `documentLanguageIds`, `filePatterns`, `capabilities`, and `readiness`, but configs without those hints must still parse.

## Implementation Checklist

- [ ] Replace optional TypeScript-only LSP-IO metadata with full-registry metadata coverage.
- [ ] Add LSP-IO registry coverage tests so every server entry exposes SDL-MCP metadata.
- [ ] Add conservative SDL-MCP config fragment export and direct config merge/write support.
- [ ] Add `lsp-io export sdl-mcp` with `--include-missing`, `--write-config`, and `--validate-launch`.
- [ ] Respect `.gitignore` in language detection.
- [ ] Split detection into recommendation-grade and all-evidence profiles.
- [ ] Filter ignored/temp/tool-cache and fixture-only evidence out of install recommendations.
- [ ] Extend SDL-MCP config schema with optional LSP metadata hints.
- [ ] Add LSP-derived language packs so configured LSP servers are selectable without tree-sitter adapters.
- [ ] Replace SDL-MCP’s non-TypeScript rejection with generic capability-driven LSP provider execution.
- [ ] Add LSP-native document-symbol ingestion and skipped/failed/completed provider-run recording.
- [ ] Update both repos’ docs and examples.
- [ ] Add cross-repo contract verification: generate LSP-IO SDL-MCP config, parse it with SDL-MCP schema, and run a dry-run semantic-enrichment path.

## Test Plan

- LSP-IO registry coverage:
  fail if any current registry entry lacks SDL-MCP launch metadata.
- LSP-IO export:
  installed/overridden servers emit SDL-MCP config; missing servers are skipped unless preview mode is requested; shared install roots do not duplicate broken config.
- LSP-IO detection:
  ignored `.tmp`, `.worktrees`, `.sisyphus`, `.codex`, `.claude`, build output, and fixture-only evidence do not feed install recommendations.
- SDL-MCP unit tests:
  non-TypeScript LSP servers are selected, initialized, and recorded; unsupported capabilities become skipped runs instead of hard rejection.
- SDL-MCP integration tests:
  mock LSP servers for document symbols only, diagnostics only, definition support, reference support, no useful capabilities, and failed initialize.
- Cross-repo contract:
  generate LSP-IO SDL-MCP config, parse it through SDL-MCP config schema, and run a dry-run semantic enrichment status/refresh path.
- Optional nightly/manual matrix:
  launch real installable servers in groups. Do not require all 103 real servers in PR CI because that would be too slow and toolchain-heavy.

## Assumptions

- “All LSP servers” means all current LSP-IO registry entries, not only SDL-MCP’s existing indexed languages.
- “Ready for SDL-MCP use” means end-to-end readiness once a command is resolved: LSP-IO can export the server, SDL-MCP can launch it, initialize it, run supported LSP capabilities, and record a provider result.
- Manual/toolchain servers are considered ready only when a valid command is found on `PATH` or supplied through `.lsp-io.toml`.
- LSP-IO must not fabricate runnable SDL-MCP config for missing tools unless the user explicitly asks for preview output with `--include-missing`.
