# Changelog

All notable changes to the Running Pseudo Language (RPL) compiler and ecosystem are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to the RPL Versioning Policy (`MAJOR.MINOR[+PATCH] "Codename"`).

---

## [0.2+6] "Tohtlane" — 2026-10-12

### Added
- **Core Timing Primitives (`sleep`, `time_now`):** Added built-in thread sleep (`sleep(ms: Int)`) and millisecond timestamp retrieval (`time_now() -> Int`) across typechecker, C99 backend (`Sleep` / `nanosleep`, `GetTickCount64` / `clock_gettime`), and Cranelift JIT (`rpl_jit_sleep`, `rpl_jit_time_now`).
- **Semantic Duration Helpers (`seconds`, `millis`):** Added unit conversion functions (`seconds(s) -> s * 1000`, `millis(ms) -> ms`) with full pipeline compatibility (`50 |> millis |> sleep`, `2 |> seconds |> sleep`).
- **LSP Tooling & Hover Support:** Real-time hover signatures, parameters, and documentation for `sleep`, `time_now`, `seconds`, and `millis` in `rpl_lsp`.
- **Official Test Fixture:** Added `tests/fixtures/time_and_sleep.rpl` validating time monotonicity, sleep delays, converters, and pipe execution.
- **Automated CI/CD Release Notes Pipeline:** Automated release notes extraction from `CHANGELOG.md` in `.github/workflows/release.yml`.

### Fixed
- **Release Packaging Path Synchronization:** Updated distribution archive bundling in `release.yml` to package `docs/guide/LANGUAGE_GUIDE.md` following documentation taxonomy stratification.

---

## [0.2+5] "Tohtlane" — 2026-10-12

### Added
- **Core `while` Loop Statement:** Implemented `while <cond>:` loop across AST, recursive descent parser, typechecker, C99 backend, and Cranelift JIT SSA block lowering. Supports both labeled `end while` and plain `end`.
- **`assert(cond: Bool)` Built-in Intrinsic:** Integrated runtime assertion intrinsic verifying invariants in C99 (`rpl_assert`) and Cranelift JIT (`rpl_jit_assert`), with static boolean argument typechecking.
- **`len(val: String) -> Int` Built-in Intrinsic:** Added string byte length calculation intrinsic across both execution backends (`rpl_str_len` in C99 and `rpl_jit_str_len` in Cranelift JIT) with pipeline support (`s |> len`).
- **LSP Tooling & Hover Support:** Live hover signatures and documentation for `while`, `assert`, and `len` in `rpl_lsp` Language Server Protocol daemon.
- **Official Test Fixture:** Added `tests/fixtures/while_assert_len.rpl` validating while loops, assert, and len operations.

---

## [0.2+4] "Tohtlane" — 2026-10-01

### Added
- **Two-Tier I/O Architecture:** Native input and file processing system spanning Layer 1 convenience pseudocode functions (`input`, `read_file`, `write_file`, `append_file`) and Layer 2 long-lived system stream handles (`File`, `open_file`, `read_line`, `write_line`, `close_file`).
- **Static Affine Ownership Enforcement:** Compile-time linear/affine tracking that guarantees `close_file` consumes handle ownership, statically rejecting use-after-close bugs with actionable diagnostics.
- **Dual-Backend Runtime Bridges:** Complete C99 runtime implementations in `rpl_runtime.h` (`rpl_file_t`, `rpl_read_file`, `rpl_write_file`, etc.) and ABI-compatible Cranelift JIT runtime symbols (`rpl_jit_*`, `JitFile`).
- **IDE & LSP Tooling Integration:** Real-time hover signatures and semantic token highlighting for `File` and all eight I/O operations across Zed, VS Code, and Antigravity IDE.
- **Automated Test Fixtures & Regressions:** Expanded `tests/fixtures/` suite with `io_complete.rpl`, `control_flow.rpl`, `functions_pipeline.rpl`, and negative compile-fail test `io_use_after_close.rpl`, wired to automated CLI regression test suite.

### Changed
- **Compiler Operating Guidelines (`AGENTS.md`):** Established ironclad invariant requiring fresh binary rebuild (`cargo build --bin rpl`) immediately after Rust workspace tests before executing any `.rpl` files.

---

## [0.2+3] "Tohtlane" — 2026-10-01

### Added
- **Native Zed Editor Extension (`wasm32-wasip2`):** Compiled WebAssembly Component Model extension in `editors/zed` supporting native language registration, automatic `rpl lsp` launching, and semantic token coloring.
- **LSP Semantic Token Rules:** Declarative semantic token mapping in `editors/zed/languages/rpl/semantic_token_rules.json` aligning keywords, types, variables, functions, strings, numbers, operators, and comments.
- **Priority-Tiered Strategic Roadmap:** Restructured `docs/ROADMAP.md` into an agile priority matrix (P0 Base through P4 Deferred Parking Lot).

### Changed
- **LSP Modularization:** Decoupled Language Server state management and protocol handlers into dedicated `crates/rpl_lsp/src/backend.rs` module.
- **Git Commit Protocol:** Replaced rigid HIDC trailer schema with pragmatic "Not What, But Why" (NWBW) commit protocol in `AGENTS.md`, featuring optional LLM context blocks.

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
