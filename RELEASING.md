# Releasing LSP-IO

This project is currently pre-release. Before cutting a public release:

1. Update `CHANGELOG.md`.
2. Run `cargo test`.
3. Run `cargo check`.
4. Run `npm run build` from `gui/`.
5. Build GUI installers with `cargo tauri build`.
6. Smoke-test `lsp-io status` and at least one managed install/remove flow in a disposable cache.

Expected release artifacts:

- CLI archives named `lsp-io-vX.Y.Z-<target>.tar.gz` or `.zip`.
- GUI installers named with the Tauri product name `LSP-IO`.
