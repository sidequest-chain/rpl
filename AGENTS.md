# AGENTS.md — Autonomous Agent Operating Guidelines (RPL Project)

This document establishes operational boundaries, engineering standards, and execution workflows for all autonomous coding agents (Claude Code, Cursor, Zed AI, and local LLM agents) working inside the RPL codebase.

---

## 1. Core Principles and Work Culture

1. **Detail Preservation Imperative:**
   * Never remove content, technical specs, or examples from Markdown files or documentation to make them "leaner" or more concise.
   * Preserve full technical depth, code samples, tables, and architectural constraints.
   * Documentation must only be expanded, clarified, or appended to, never truncated.

2. **Surgical Code Modifications:**
   * Only modify code that is strictly required to implement the requested feature or fix the specified bug.
   * Do not remove working debug or logging statements (`println!`, `eprintln!`, `dbg!`).
   * Do not refactor functional code or rewrite components merely for style unless explicitly commanded.
   * Do not alter existing code comments unless the underlying implementation changed and rendered the comment factually false.

3. **Language Rules:**
   * All code comments, function documentation (docstrings), parser errors, and diagnostic output must be in **English**.
   * Conversational dialogue with the user remains in **Estonian** unless explicitly directed otherwise.

---

## 2. Workspace Architecture

The repository is structured as a modular Rust workspace:

```text
/
├── AGENTS.md               # Agent guidelines and operational boundaries
├── PROJECT_SPEC.md         # RPL language specification and grammar
├── Cargo.toml              # Root workspace manifest
├── crates/
│   ├── rpl_lexer/          # Tokenization engine (.rpl source to token stream)
│   ├── rpl_parser/         # Recursive Descent + Pratt parser producing AST
│   ├── rpl_ast/            # Concrete Abstract Syntax Tree data models
│   ├── rpl_typechecker/    # Type verification, Trit semantics, and move checking
│   ├── rpl_codegen_c/      # Phase 1: High-performance C99 transpiler backend
│   └── rpl_cli/            # CLI binary interface (rpl run, rpl build, rpl check)
└── tests/
    └── fixtures/           # Official .rpl test files and grammar targets
```

---

## 3. Compiler Implementation Constraints

1. **Grammar and Syntax Compliance:**
   * Scopes open with `:` and close exclusively with `Token::End`.
   * Structural curly braces `{}` and semicolons `;` must not be parsed or emitted under any circumstances.
   * String interpolation parsing must strictly recognize `$var` and `$(expr)`.

2. **Lexer Behavior (`rpl_lexer`):**
   * Newlines (`\n`) carry semantic significance as statement terminators outside open parentheses. The lexer must emit `Token::Newline` appropriately.
   * Indentation must not trigger parser panics; structural scoping is determined solely by matching block openers with `Token::End`.

3. **Ternary Verification (`rpl_typechecker`):**
   * The `Trit` type supports three variants: `true`, `false`, and `unknown`.
   * Pattern matching over `Trit` must enforce exhaustiveness at compile time, rejecting missing cases.

4. **Diagnostic Reporting:**
   * Compiler diagnostics must provide line, column, and span coordinates.
   * Error messages must deliver actionable guidance (e.g., `"SyntaxError: Expected 'end' to close block opened at line 24:5"`).

---

## 4. Verification Workflow

Before completing any task, an agent must run and satisfy this verification pipeline:

1. **Compilation Check:**
   ```bash
   cargo check --workspace
   ```
2. **Workspace Test Suite:**
   ```bash
   cargo test --workspace
   ```
3. **Linter and Static Analysis:**
   ```bash
   cargo clippy --workspace -- -D warnings
   ```

If any step fails, the agent must document the root cause before applying the minimal corrective diff.
