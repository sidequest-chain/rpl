# CODE_MAP — RPL Architecture & Codebase Navigation Index

> **Purpose:** This document is an optimized index and structural map for LLM agents (Claude Code, Cursor, Zed AI, Antigravity) and human developers to rapidly understand module boundaries, data flows, type definitions, and invariants without expensive full-repo scans.

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
    - `Type`: `Int`, `Float`, `String`, `Bool`, `Trit`, `Void`, `List(Box<Type>)`, `Map(Box<Type>, Box<Type>)`, `Custom(String)`, `Fn(Vec<Type>, Box<Type>)`.
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
  - `src/error.rs`:
    - `ParserError`: Actionable diagnostic messages with span coordinates.
  - `tests/parser_tests.rs`: Tests covering grammar constructs and language spec examples.

---

### 2.4 `rpl_typechecker`
- **Location:** `crates/rpl_typechecker/`
- **Role:** Semantic validation, type inference, Kleene 3-value logic consistency, and pattern match exhaustiveness.
- **Key Modules & Files:**
  - `src/lib.rs`:
    - `check_program(program: &Program) -> Result<(), Vec<TypeError>>`: Main entrypoint.
  - `src/checker.rs`:
    - `TypeChecker`: AST traversal checking variable declarations, mutations, types of expressions, and control structures.
    - Exhaustiveness checking for `Trit` pattern matches (ensures `true`, `false`, and `unknown` are all handled).
  - `src/env.rs`:
    - `Environment`: Scoped symbol tables tracking variable types, mutability (`mut`), and initialization / move states (`Uninit`, `Valid`, `Moved`).
    - Built-in functions seeding (`print`, `println`, `len`, `assert`).
  - `src/error.rs`:
    - `TypeError`: Detailed semantic errors (`TypeMismatch`, `UndefinedVariable`, `CannotMutateImmutable`, `NonExhaustiveMatch`, `UseAfterMove`).
  - `tests/typechecker_tests.rs`: Tests for Kleene truth tables, immutability, and trit match coverage.

---

### 2.5 `rpl_codegen_c`
- **Location:** `crates/rpl_codegen_c/`
- **Status:** ✅ **Complete (Phase 1)** — C99 transpiler backend emitting clean, optimized C99 code.
- **Key Modules & Files:**
  - `src/lib.rs`: `generate_c(program: &Program) -> Result<String, CodegenError>` entrypoint.
  - `src/codegen.rs`: `CGenerator` translating declarations, functions, expressions, and control flow.
  - `src/runtime.rs`: `RPL_RUNTIME_H` header with Kleene ternary logic (`rpl_trit_t`), output functions, and string interpolation helpers.
  - `src/types.rs`: C type mapping functions (`to_c_type`, `to_c_return_type`).
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
- **Status:** Phase 2 Target (In-memory Cranelift JIT engine for sub-millisecond execution and REPL without external C toolchain).
- **Architecture & Roadmap:** See [ROADMAP.md](ROADMAP.md) for phased execution strategy and self-hosting bootstrap milestones.

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
