# CODE_MAP — RPL Architecture & Codebase Navigation Index

> **Purpose:** This document is an optimized index and structural map for LLM agents (Claude Code, Cursor, Zed AI, Antigravity) and human developers to rapidly understand module boundaries, data flows, type definitions, and invariants without expensive full-repo scans.
>
> **Companion Guides:**
> - [docs/COMPILER_CAPABILITIES.md](file:///d:/Dev/rpl/docs/COMPILER_CAPABILITIES.md) — Authoritative implementation status, 100% working feature matrix, and partner LLM extension prompts.
> - [docs/LANGUAGE_GUIDE.md](file:///d:/Dev/rpl/docs/LANGUAGE_GUIDE.md) — Hands-on tutorial and developer reference.
> - [docs/PROJECT_SPEC.md](file:///d:/Dev/rpl/docs/PROJECT_SPEC.md) — Long-term technical specification and formal grammar.
> - [docs/ROADMAP.md](file:///d:/Dev/rpl/docs/ROADMAP.md) — Priority-tiered development roadmap (P0–P4).

---

## Quick Navigation Index
1. [Pipeline & Data Flow Overview](#1-pipeline--data-flow-overview)
2. [Crates Index & Responsibilities](#2-crates-index--responsibilities)
   - [2.1 rpl_ast (Abstract Syntax Tree)](#21-rpl_ast)
   - [2.2 rpl_lexer (Tokenizer & Lexical Analysis)](#22-rpl_lexer)
   - [2.3 rpl_parser (Recursive Descent & Pratt Parser)](#23-rpl_parser)
   - [2.4 rpl_typechecker (Semantic & Kleene Logic Validator)](#24-rpl_typechecker)
   - [2.5 rpl_codegen_c (Phase 1 C99 Transpiler Backend)](#25-rpl_codegen_c)
   - [2.6 rpl_cli (CLI Interface)](#26-rpl_cli)
   - [2.7 rpl_codegen_cranelift (Phase 2 Cranelift JIT)](#27-rpl_codegen_cranelift)
   - [2.8 rpl_lsp (Language Server Protocol)](#28-rpl_lsp)
3. [Language Invariants & Grammar Rules](#3-language-invariants--grammar-rules)
4. [Frequent Development Paths (Extension Cheatsheet)](#4-frequent-development-paths)
5. [Verification Commands](#5-verification-commands)

---

## 1. Pipeline & Data Flow Overview

```text
       RPL Source Code (.rpl)
                │
                ▼
  crates/rpl_lexer (Logos tokenizer, span line/col tracking, string interpolation)
                │ Stream of (Token, Span)
                ▼
  crates/rpl_parser (Recursive Descent for Stmts + Pratt Parsing for Exprs)
                │ Program AST (rpl_ast)
                ▼
  crates/rpl_typechecker (Type checking, Kleene trit algebra, move semantics)
                │ Validated Program AST
                ├─────────────────────────────────────────────┐
                ▼                                             ▼
  crates/rpl_codegen_c (Phase 1: C99 Transpiler)   crates/rpl_codegen_cranelift (Phase 2: JIT)
                │ Output C99 code (.c / .h)                   │ In-Memory Machine Code
                ▼                                             ▼
  Target Compiler (Clang / GCC / MSVC) -> Native Binary   Instant In-Memory Execution
```

---

## 2. Crates Index & Responsibilities

### 2.1 `rpl_ast`
- **Location:** `crates/rpl_ast/`
- **Role:** Pure data structures defining the strongly-typed Abstract Syntax Tree. Free of compiler logic or parser side-effects.
- **Key Modules & Files:**
  - `src/lib.rs`: Root re-exports of core AST types.
  - `src/span.rs`:
    - `Span`: Byte offset bounds `[start..end]` and 1-based `(start_line, start_col, end_line, end_col)` diagnostics coordinates.
  - `src/literal.rs`:
    - `Literal`: `Int(i64)`, `Float(f64)`, `String(String)`, `Bool(bool)`, `Trit(TritValue)`.
    - `TritValue`: 3-valued Kleene logic variants: `True`, `False`, `Unknown`. Implements ternary `and`, `or`, `not`.
  - `src/types.rs`:
    - `Type`: `Int`, `Float`, `String`, `Bool`, `Trit`, `File` (opaque stream handle), `Void`, `List(Box<Type>)`, `Map(Box<Type>, Box<Type>)`, `Custom(String)`, `Fn(Vec<Type>, Box<Type>)`.
  - `src/op.rs`:
    - `BinaryOp`: Arithmetic (`+`, `-`, `*`, `/`, `%`), Comparison (`==`, `!=`, `<`, `<=`, `>`, `>=`), Logical (`and`, `or`), Bitwise/Shift, Pipe (`|>`), Range (`..`).
    - `UnaryOp`: Negation (`-`), Logical Not (`not`), Bitwise Not.
  - `src/expr.rs`:
    - `Expr`: Recursive expression tree enum: `Literal`, `Ident`, `Binary`, `Unary`, `Call`, `MemberAccess`, `Index`, `Lambda`, `ListLiteral`, `MapLiteral`, `InterpolatedString`, etc.
    - `InterpolationFragment`: `Literal(String)` and `Expr(Box<Expr>)` for `$var` / `$(expr)`.
  - `src/stmt.rs`:
    - `Program`: Root node containing `Vec<Stmt>`.
    - `Stmt`: `VarDecl`, `Assign`, `FnDecl`, `If`, `Match`, `For`, `While`, `Return`, `Expr`, `Spawn`, etc.
    - `Pattern`: Match case patterns including literals, identifiers, wildcards, and Trit variants.
  - `src/error.rs`: `AstError` error definitions via `thiserror`.
  - `tests/ast_tests.rs`: Comprehensive unit tests for AST nodes and truth tables.

---

### 2.2 `rpl_lexer`
- **Location:** `crates/rpl_lexer/`
- **Role:** Converts raw UTF-8 `.rpl` source code into `(Token, Span)` pairs with line/column tracking.
- **Key Modules & Files:**
  - `src/lib.rs`:
    - `RplLexer<'a>`: Iterator wrapping `logos::Lexer`.
    - Tracks bracket depth (`()`, `[]`) to ignore insignificant newlines within arguments while emitting semantic `Token::Newline` statements terminators at depth 0.
  - `src/token.rs`:
    - `Token`: 28 reserved keywords (`fn`, `let`, `mut`, `if`, `else`, `match`, `for`, `in`, `parallel`, `spawn`, `return`, `end`, `true`, `false`, `unknown`, `trit`, etc.), literals, symbols, and operators.
  - `src/interpolation.rs`:
    - `split_interpolation`: Detects and segments `$ident` and `$(expr)` within interpolated strings.
  - `src/error.rs`:
    - `LexerError`: Structured syntax errors with spans (`UnterminatedString`, `InvalidNumberLiteral`, `UnexpectedCharacter`).
  - `tests/lexer_tests.rs`: Test suites verifying keyword tokenization, newline filtering, and string interpolation.

---

### 2.3 `rpl_parser`
- **Location:** `crates/rpl_parser/`
- **Role:** Parses the token stream into `rpl_ast::Program`.
- **Parsing Strategy:**
  - **Statements & Declarations:** Recursive Descent.
  - **Expressions:** Pratt parsing (Precedence Climbing) for mathematical, logical, and pipe (`|>`) operators.
- **Key Modules & Files:**
  - `src/lib.rs`:
    - `Parser<'a>`: Lookahead token buffer (`peek`, `peek_next`, `advance`, `expect_token`).
    - `parse_program(source: &str) -> Result<Program, ParserError>`: Main entrypoint.
    - `parse_expression(source: &str) -> Result<Expr, ParserError>`: Expression entrypoint.
  - `src/expr.rs`:
    - `Precedence` hierarchy: `Assignment` < `Pipe` < `LogicalOr` < `LogicalAnd` < `Equality` < `Comparison` < `Term` < `Factor` < `Unary` < `Call/Index/Member`.
  - `src/stmt.rs`:
    - Scoping enforcement: Blocks must start with `:` and terminate with `Token::End`.
    - Type parsing including `Type::File` recognition.
  - `src/error.rs`:
    - `ParserError`: Actionable diagnostic messages with span coordinates.
  - `tests/parser_tests.rs`: Tests covering grammar constructs and language spec examples.

---

### 2.4 `rpl_typechecker`
- **Location:** `crates/rpl_typechecker/`
- **Role:** Semantic validation, type inference, Kleene 3-value logic consistency, pattern match exhaustiveness, and affine move tracking.
- **Key Modules & Files:**
  - `src/lib.rs`:
    - `check_program(program: &Program) -> Result<(), Vec<TypeError>>`: Main entrypoint.
  - `src/checker.rs`:
    - `TypeChecker`: AST traversal checking variable declarations, mutations, types of expressions, control structures, and affine move enforcement for `close_file`.
    - Exhaustiveness checking for `Trit` pattern matches (ensures `true`, `false`, and `unknown` are all handled).
  - `src/env.rs`:
    - `Environment`: Scoped symbol tables tracking variable types, mutability (`mut`), and initialization / move states (`Uninit`, `Valid`, `Moved`).
    - Built-in functions seeding (`print`, `println`, `len`, `assert`, `input`, `read_file`, `write_file`, `append_file`, `open_file`, `read_line`, `write_line`, `close_file`).
  - `src/error.rs`:
    - `TypeError`: Detailed semantic errors (`TypeMismatch`, `UndefinedVariable`, `CannotMutateImmutable`, `NonExhaustiveMatch`, `UseOfMovedValue`).
  - `tests/typechecker_tests.rs`: Tests for Kleene truth tables, immutability, trit match coverage, and affine use-after-close checks.

---

### 2.5 `rpl_codegen_c`
- **Location:** `crates/rpl_codegen_c/`
- **Status:** ✅ **Complete (Phase 1)** — C99 transpiler backend emitting clean, optimized C99 code with Two-Tier I/O.
- **Key Modules & Files:**
  - `src/lib.rs`: `generate_c(program: &Program) -> Result<String, CodegenError>` entrypoint.
  - `src/codegen.rs`: `CGenerator` translating declarations, functions, expressions, and control flow.
  - `src/runtime.rs`: `RPL_RUNTIME_H` header with Kleene ternary logic (`rpl_trit_t`), output functions, Two-Tier I/O functions (`rpl_input`, `rpl_read_file`, `rpl_write_file`, `rpl_append_file`, `rpl_open_file`, `rpl_read_line`, `rpl_write_line`, `rpl_close_file`), and string interpolation helpers.
  - `src/types.rs`: C type mapping functions (`to_c_type`, `to_c_return_type`, `Type::File` -> `rpl_file_t`).
  - `src/error.rs`: `CodegenError` definitions via `thiserror`.
  - `tests/codegen_tests.rs`: Comprehensive test suite verifying C generation and native compilation with host C compiler.

---

### 2.6 `rpl_cli`
- **Location:** `crates/rpl_cli/`
- **Status:** ✅ **Complete (Phase 1)** — CLI binary driver providing `rpl check`, `rpl build`, and `rpl run`.
- **Key Modules & Files:**
  - `src/main.rs`: Clap-based command line interface detecting host C compilers (`clang`, `gcc`, `zig cc`, `cc`, `cl.exe`).
  - `tests/cli_tests.rs`: End-to-end integration tests for `check`, `build --emit-c`, and `run`.

---

### 2.7 `rpl_codegen_cranelift`
- **Location:** `crates/rpl_codegen_cranelift/`
- **Status:** ✅ **Complete (Phase 2)** — In-memory Cranelift JIT engine delivering sub-millisecond execution without external C toolchains.
- **Key Modules & Files:**
  - `src/lib.rs`: `run_program(program: &Program) -> Result<i64, CodegenCraneliftError>` entrypoint.
  - `src/compiler.rs`: AST lowering into Cranelift IR, Kleene ternary logic in CPU registers, struct stack slot allocation, control flow, functions, loops, string interpolation lowering, and Two-Tier I/O direct dispatch.
  - `src/jit.rs`: `JITCompiler` native host ISA builder, JIT module management, and memory execution.
  - `src/runtime.rs`: Native C-ABI runtime helper functions (`rpl_jit_print_*`, `rpl_jit_str_concat`, `rpl_jit_input`, `rpl_jit_read_file`, `rpl_jit_write_file`, `rpl_jit_append_file`, `rpl_jit_open_file`, `rpl_jit_read_line`, `rpl_jit_write_line`, `rpl_jit_close_file`, `JitFile`) and symbol table registration.
  - `src/types.rs`: Cranelift type translation (`rpl_to_cl_type`, `Type::File` -> pointer) and memory layout computation (`compute_struct_layout`).
  - `src/error.rs`: `CodegenCraneliftError` definitions via `thiserror`.
  - `tests/jit_tests.rs`: Unit and integration test suite verifying JIT arithmetic, Kleene ternary logic truth tables, struct access, loops, Two-Tier I/O, and `examples/reaktor.rpl`.
- **Architecture & Roadmap:** See [ROADMAP.md](ROADMAP.md) for phased execution strategy and self-hosting bootstrap milestones.

---

### 2.8 `rpl_lsp`
- **Location:** `crates/rpl_lsp/`
- **Status:** ✅ **Complete** — Language Server Protocol (LSP) daemon powering IDE tooling (`rpl lsp`) across Zed, VS Code, and Antigravity IDE.
- **Key Modules & Files:**
  - `src/lib.rs`: `tower-lsp` LanguageServer implementation, `compute_diagnostics` (syntax and Kleene ternary type checking), LSP 3.17 semantic tokens (including `File` and I/O functions), and hover documentation.
  - `tests/lsp_tests.rs`: Comprehensive test suite verifying diagnostic generation (`[- - -]` syntax and `[+ - -]` type errors), clean document reporting, and hover tooltips for types and built-in I/O functions.
- **Editor Extensions & Setup:** See [IDE_SETUP.md](IDE_SETUP.md), `editors/code/` (VS Code / Antigravity IDE), and `editors/zed/` (native Zed extension).
- **Editor Extensions & Setup:** See [IDE_SETUP.md](IDE_SETUP.md), `editors/code/` (VS Code / Antigravity IDE), and `editors/zed/` (native Zed extension).

---

## 3. Language Invariants & Grammar Rules

When reading or modifying RPL compiler code, keep these invariants strictly intact:

| Invariant | Description |
| :--- | :--- |
| **No `{}` or `;`** | Curly braces `{}` and semicolons `;` are strictly prohibited in RPL grammar and must never be accepted by the lexer or parser. |
| **Block Delimiters** | Blocks start with `:` and close exclusively with `end` (`Token::End`). |
| **Indentation-Agnostic** | Scoping is governed solely by `:` and `end`. Indentation (spaces, tabs, or zero indent) is purely stylistic and ignored by the compiler. |
| **Newlines as Terminators** | `\n` acts as a statement terminator at depth 0, but is ignored inside parentheses `(...)` and brackets `[...]`. |
| **Trit 3-State Logic** | `Trit` has 3 states: `true`, `false`, `unknown`. Pattern matches over `Trit` MUST be exhaustive. |
| **String Interpolation** | Strictly recognizes `$var` and `$(expr)`. |
| **Tiered Git Protocol** | All commits must follow Tier 1 (Lite NWBW) or Tier 2 (Full HIDC) via `commit_msg.txt`. |
| **Versioning Policy** | Strict format `MAJOR.MINOR+PATCH "Codename"`, folklore codenames (Kreutzwald), package schema lock, and trinary status vectors `[Syntax/Parser . Typechecker . Codegen]`. |

---

## 4. Frequent Development Paths

### Adding a new keyword:
1. `crates/rpl_lexer/src/token.rs`: Add variant to `Token` enum with logos pattern.
2. `crates/rpl_parser/src/stmt.rs` or `expr.rs`: Handle token in parser dispatch.
3. Update lexer and parser unit tests.

### Adding an AST expression / statement:
1. `crates/rpl_ast/src/expr.rs` or `stmt.rs`: Add node variant with `Span`.
2. `crates/rpl_parser/src/expr.rs` or `stmt.rs`: Add parsing rule with precedence.
3. `crates/rpl_typechecker/src/checker.rs`: Add typecheck rule in `check_expr` or `check_stmt`.
4. Add corresponding test in `crates/rpl_typechecker/tests/`.

---

## 5. Verification Commands

Always run this pipeline before proposing or completing any modifications:

```bash
# 1. Quick compilation check
cargo check --workspace

# 2. Complete workspace test suite
cargo test --workspace

# 3. Strict clippy analysis
cargo clippy --workspace -- -D warnings
```
