# IDE Setup & Integration Guide (RPL Language Server)

This guide explains how to configure **Zed**, **Antigravity IDE**, and **Visual Studio Code** to use the official Running Pseudo Language (RPL) Language Server Protocol (`rpl lsp`) for real-time syntax checking, Kleene ternary type diagnostics, and hover documentation.

---

## 1. Prerequisites

Ensure the `rpl` compiler executable is installed and accessible in your system `PATH`:

```powershell
# Verify installation
rpl --version
# Output: rpl 0.2+2 "Tohtlane" ...
```

The language server is built directly into the `rpl` binary and invoked via:
```powershell
rpl lsp
```

---

## 2. Zed Editor Setup

Zed supports custom Language Servers natively via its settings file.

### Step 1: Open Zed Settings
Open Zed and press `Ctrl+,` (or `Cmd+,` on macOS), or edit your configuration directly:
- **Windows:** `%APPDATA%\Zed\settings.json`
- **Linux / macOS:** `~/.config/zed/settings.json`

### Step 2: Add Language Server Configuration
Add the following configuration to your `settings.json`:

```json
{
  "languages": {
    "RPL": {
      "language_servers": ["rpl-lsp"]
    }
  },
  "lsp": {
    "rpl-lsp": {
      "binary": {
        "path": "rpl",
        "arguments": ["lsp"]
      }
    }
  }
}
```

If `rpl` is not in your system PATH, specify the absolute path to the binary:
```json
{
  "lsp": {
    "rpl-lsp": {
      "binary": {
        "path": "C:\\Program Files\\RunningPseudoLanguage\\rpl.exe",
        "arguments": ["lsp"]
      }
    }
  }
}
```

---

## 3. Visual Studio Code & Antigravity IDE Setup

The RPL repository includes an extension in `editors/code/` providing:
1. **TextMate Syntax Highlighting:** Keywords (`fn`, `match`, `type`, `let`, `mut`), Ternary logic types (`Trit`, `true`, `false`, `unknown`), and string interpolation (`$var`, `$(expr)`).
2. **Language Configuration:** Automatic bracket pairing and indentation after `:` and outdent on `end`.
3. **LSP Client:** Spawns `rpl lsp` in the background and delivers real-time error markers.

### Step 1: Install Extension Dependencies
Open a terminal in the extension folder and install client dependencies:

```powershell
cd editors/code
npm install
```

### Step 2: Link Extension into Editor

Link or copy the `editors/code` folder into your editor's extension directory:

#### For Antigravity IDE:
* **Windows (PowerShell as Administrator or with Developer Mode):**
  ```powershell
  New-Item -ItemType SymbolicLink -Path "$HOME\.antigravity-ide\extensions\rpl" -Target "D:\Dev\rpl\editors\code"
  ```
* **Or simple copy without symlinks:**
  ```powershell
  Copy-Item -Recurse -Path "D:\Dev\rpl\editors\code" -Destination "$HOME\.antigravity-ide\extensions\rpl"
  ```

#### For VS Code:
* **Windows (PowerShell):**
  ```powershell
  New-Item -ItemType SymbolicLink -Path "$HOME\.vscode\extensions\rpl" -Target "D:\Dev\rpl\editors\code"
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
