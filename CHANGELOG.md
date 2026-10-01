# Changelog

All notable changes to the Running Pseudo Language (RPL) compiler and ecosystem are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to the RPL Versioning Policy (`MAJOR.MINOR[+PATCH] "Codename"`).

---

## [0.2+3] "Tohtlane" — 2026-10-01

### Added
- **Native Zed Editor Extension (`wasm32-wasip2`):** Compiled WebAssembly Component Model extension in `editors/zed` supporting native language registration, automatic `rpl lsp` launching, and semantic token coloring.
- **LSP Semantic Token Rules:** Declarative semantic token mapping in `editors/zed/languages/rpl/semantic_token_rules.json` aligning keywords, types, variables, functions, strings, numbers, operators, and comments.
- **Local Deployment Automation:** PowerShell and batch installation scripts (`tools/install.ps1`, `tools/install.cmd`) building release binaries and safely deploying directly to `C:\Program Files\RunningPseudoLanguage` with UAC elevation support.
- **Priority-Tiered Strategic Roadmap:** Restructured `docs/ROADMAP.md` into an agile priority matrix (P0 Base through P4 Deferred Parking Lot).

### Changed
- **LSP Modularization:** Decoupled Language Server state management and protocol handlers into dedicated `crates/rpl_lsp/src/backend.rs` module.
- **Git Commit Protocol:** Replaced rigid HIDC trailer schema with pragmatic "Not What, But Why" (NWBW) commit protocol in `AGENTS.md`, featuring optional LLM context blocks.
- **Local Tools Ignore:** Added `tools/` directory to `.gitignore` to prevent workstation automation scripts from leaking upstream.

---

## [0.2+2] "Tohtlane" — 2026-10-01

### Fixed
- **Bitwise Logic Operators:** Aligned natural-language operators `and`, `or`, `not` across both boolean/trit logic and integer bitwise operations.
- **C99 Trit Code Generation:** Corrected signed 8-bit Trit representation and branch lowering in `rpl_codegen_c`.

### Added
- **Embedded LSP Semantic Tokens Provider:** Integrated LSP 3.17 semantic tokens provider into `crates/rpl_lsp` delivering rich syntax highlighting without external Tree-sitter requirements.

---

## [0.2+1] "Tohtlane" — 2026-09-30

### Added
- **Zero-Dependency Language Server (`rpl lsp`):** Embedded Language Server Protocol daemon directly into the monolithic `rpl` CLI binary supporting diagnostics and hover documentation.
- **Automated CI/CD Release Pipeline:** GitHub Actions workflow automatically building, packaging, and publishing multi-platform release archives for Windows and Linux on tag push.
- **VS Code Extension Client:** Clean Language Client implementation in `editors/code` launching embedded `rpl lsp` over stdio with TextMate fallback grammar.
- **IDE Setup Documentation:** Comprehensive developer onboarding guide in `docs/IDE_SETUP.md`.

### Changed
- **Versioning Policy Formalization:** Standardized monotonic patch counters (`+1`, `+2`, `+x`) and registered Kreutzwald folklore codenames in `AGENTS.md`.

---

## [0.2] "Tohtlane" — 2026-09-30

### Added
- **In-Memory Cranelift JIT Backend (`crates/rpl_codegen_cranelift`):** Sub-millisecond machine code compilation and execution directly in memory via `cranelift-jit` and `cranelift-module`.
- **Default JIT Execution:** `rpl run <file.rpl>` executes via Cranelift JIT by default, with `--via-c` providing explicit opt-in to portable C99 compilation.
- **Native Host Calling Conventions:** Seamless ABI negotiation for `WindowsFastcall` on Windows and `SystemV` on Linux.
- **Runtime FFI Bridge:** Native bridge interconnecting JIT machine code with RPL runtime helper functions (`rpl_print_str`, `rpl_print_trit`, `rpl_print_i64`).

---

## [0.1] "Puulane" — 2026-09-30

### Added
- **High-Performance C99 Transpiler (`crates/rpl_codegen_c`):** Clean, human-readable C99 code emission for variables, functions, loops, structs, and Kleene ternary logic.
- **Unified CLI Driver (`crates/rpl_cli`):** Command-line interface offering `rpl run`, `rpl build`, and `rpl check` with automatic host C compiler detection (`clang`, `gcc`, `cl.exe`).
- **Header-Only Runtime Library:** `rpl_runtime.h` managing scoped heap buffers, string interpolation, and standard I/O.
- **Comprehensive Compiler Frontend:**
  - `crates/rpl_ast`: Concrete Abstract Syntax Tree data models.
  - `crates/rpl_lexer`: Logos-based tokenizer with newline semantics and string interpolation.
  - `crates/rpl_parser`: Recursive Descent + Pratt parser with precedence climbing and pipe operator `|>`.
  - `crates/rpl_typechecker`: Semantic type analyzer enforcing Kleene 3-state logic and affine move checks.
- **European Union Public Licence (EUPL-1.2):** Official project licensing and governance.
