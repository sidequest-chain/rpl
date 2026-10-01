# RPL Language Extension for Zed

Official Zed extension providing native language support, filetype registration (`.rpl`), and Language Server Protocol (LSP) integration for **Running Pseudo Language (RPL)**.

## Structure
- `extension.toml`: Extension manifest declaring language `RPL` and language server `rpl`.
- `languages/rpl/config.toml`: Language configuration defining file suffix `.rpl`, comments (`//`, `/* */`), and bracket auto-closing rules.
- `src/lib.rs`: Rust extension code compiled to WebAssembly (`wasm32-wasip2`) implementing `zed_extension_api::Extension` to spawn `rpl lsp`.
- `extension.wasm`: Precompiled WebAssembly extension binary.

## Installation in Zed

1. Open **Zed**.
2. Press `Ctrl + Shift + P` (or `Cmd + Shift + P` on macOS) to open the Command Palette.
3. Type and select: **`zed: install dev extension`**.
4. In the folder picker dialog, select this directory:
   ```text
   d:\Dev\rpl\editors\zed
   ```
5. Zed will immediately load the extension. Open any `.rpl` file (such as `examples/reaktor.rpl`) to start editing with active LSP diagnostics and hover documentation.
