# IDE Setup & Integration Guide (docs/user/IDE_SETUP.md)

> **Document Role:** Editor Tooling & LSP Configuration Guide  
> **Target Audience:** RPL Developers & Contributors (**Tooling Setup**)  
> **Active Milestone:** 0.2 "Tohtlane" (Active patch: see [VERSION](../../VERSION))  

This guide explains how to configure **Zed**, **Antigravity IDE**, and **Visual Studio Code** to use the official Running Pseudo Language (RPL) Language Server Protocol (`rpl lsp`) for real-time syntax checking, Kleene ternary type diagnostics, and hover documentation.

---

## 1. Prerequisites

Ensure the `rpl` compiler executable is installed and accessible in your system `PATH`:

```powershell
# Verify installation
rpl --version
# Output: rpl 0.2... "Tohtlane" ...
```

The language server is built directly into the `rpl` binary and invoked via:
```powershell
rpl lsp
```

---

## 2. Zed Editor Setup

RPL provides a native Zed extension in `editors/zed/` compiled to WebAssembly (`wasm32-wasip2`) that registers the `RPL` language and attaches the `rpl lsp` language server.

### Option A: Install via Command Palette (Recommended for Development)
1. Open **Zed**.
2. Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS) to open the Command Palette.
3. Type and select: **`zed: install dev extension`**.
4. In the folder picker dialog, select the repository's `editors/zed` directory:
   ```text
   <path-to-repo>\editors\zed
   ```
5. Zed immediately activates language recognition for `.rpl` files, sets up brackets and comments, and connects to `rpl lsp`.

### Option B: Automatic Installation Directory
Alternatively, copy the compiled extension into Zed's installed extensions directory:
- **Windows:** `%LOCALAPPDATA%\Zed\extensions\installed\rpl`
- **Linux / macOS:** `~/.local/share/zed/extensions/installed/rpl`

---

## 3. Visual Studio Code & Antigravity IDE Setup

The RPL repository includes an extension in `editors/code/` providing:
1. **TextMate Syntax Highlighting:** Keywords (`fn`, `match`, `type`, `let`, `mut`), Ternary logic types (`Trit`, `true`, `false`, `unknown`), and string interpolation (`$var`, `$(expr)`).
2. **Language Configuration:** Automatic bracket pairing, comment toggling (`//`, `/* */`), and scope indentation.
3. **Self-Contained LSP Client:** Zero external npm dependencies. Communicates directly with `rpl lsp` via stdio JSON-RPC.

### Installation Steps

Link or copy the `editors/code` folder into your editor's extension directory (from the repository root):

#### For Antigravity IDE:
* **Windows (PowerShell as Administrator or with Developer Mode):**
  ```powershell
  New-Item -ItemType SymbolicLink -Path "$HOME\.antigravity-ide\extensions\rpl" -Target "$PWD\editors\code"
  ```
* **Or simple copy without symlinks:**
  ```powershell
  Copy-Item -Recurse -Path "$PWD\editors\code" -Destination "$HOME\.antigravity-ide\extensions\rpl"
  ```

#### For VS Code:
* **Windows (PowerShell):**
  ```powershell
  New-Item -ItemType SymbolicLink -Path "$HOME\.vscode\extensions\rpl" -Target "$PWD\editors\code"
  ```
* **Linux / macOS:**
  ```bash
  ln -s /path/to/rpl/editors/code ~/.vscode/extensions/rpl
  ```

### Step 3: Restart or Reload Editor
Restart your editor or execute **Developer: Reload Window** from the command palette (`Ctrl+Shift+P`).

---

## 4. Verification & Diagnostic Features

Once integrated, open any `.rpl` file (such as `examples/reaktor.rpl`) to verify the integration:

1. **Syntax Highlighting:**
   - Block keywords (`fn`, `type`, `match`, `for`, `end`) highlight with distinct tokens.
   - Ternary constants (`true`, `false`, `unknown`) are formatted as language literals.
   - String interpolations (`$reaktor.id`, `$(x + 1)`) are clearly highlighted.

2. **Real-time Diagnostics:**
   - **Syntax errors:** A missing `end` or invalid token displays an inline red diagnostic prefixed with `[- - -]`.
   - **Type mismatches:** Assigning incompatible types or incomplete pattern matching displays an error prefixed with `[+ - -]`.

3. **Hover Tooltips:**
   - Hover over `Trit` to view Kleene 3-valued logic algebra documentation.
   - Hover over a function name (e.g. `arvuta_turvalisus`) to view its signature in Markdown.
