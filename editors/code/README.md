# RPL Language Support for VS Code & Antigravity IDE

Provides syntax highlighting, indentation rules, and official Language Server Protocol (LSP) diagnostics for **Running Pseudo Language (RPL)** (`.rpl` files).

## Features
- **Syntax Highlighting:** TextMate grammar supporting RPL keywords (`fn`, `match`, `type`, `let`, `mut`, `spawn`), Kleene ternary logic types (`Trit`, `true`, `false`, `unknown`), and string interpolation (`$var`, `$(expr)`).
- **Diagnostics:** Inline parsing errors (`[- - -]`) and Kleene ternary type mismatch errors (`[+ - -]`).
- **Hover Documentation:** Rich Markdown tooltips for RPL keywords, primitive types, functions, and struct definitions.

## Installation
Link or copy this directory into your editor's extensions directory:
- **VS Code:** `~/.vscode/extensions/rpl` (or `%USERPROFILE%\.vscode\extensions\rpl`)
- **Antigravity IDE:** `%USERPROFILE%\.antigravity-ide\extensions\rpl`
