# RPL Architecture Roadmap & Evolution Strategy

This document outlines the strategic roadmap for the **Running Pseudo Language (RPL)** compiler, detailing the staged pathway from early cross-platform execution to a self-hosting compiler.

---

## 1. Core Architectural Strategy: The Hybrid Approach

To balance rapid cross-platform deployment (Windows & Linux) with ultimate developer experience and native speed, RPL adopts a **Hybrid Execution Model**:

```text
                             RPL Source (.rpl)
                                    │
                     ┌──────────────┴──────────────┐
                     ▼                             ▼
       Phase 1: C99 Backend               Phase 2: Cranelift JIT
       (`rpl_codegen_c`)                  (`rpl_codegen_cranelift`)
             │                                     │
    Standard C99 Code                      In-Memory Machine Code
             │                                     │
    Host CC (Clang/GCC/MSVC)                       │
             │                                     │
   Standalone Native Binary              Instant Sub-ms Execution
   (`rpl build` for Win/Linux)           (`rpl run` / REPL / Dev)
```

### Why this hybrid model?
1. **Immediate Cross-Platform Reliability (Windows + Linux):**
   * Emitting high-performance, standard C99 abstracts away OS differences, system ABIs, and platform-specific linkers.
   * Complex features (string interpolation, dynamic arrays, hash maps, and deterministic scope cleanup) can be delivered cleanly and portably using C99 and standard libc.
2. **Instant Developer Feedback:**
   * Adding Cranelift JIT in Phase 2 removes the overhead of spawning external C compilers during daily development, providing sub-millisecond in-memory execution for `rpl run` and testing.
3. **The Bootstrap Vehicle for Self-Hosting:**
   * History has shown that almost every successful self-hosting language (C++, Nim, early Rust/C) bootstrapped through C before compiling directly to bare-metal. The C99 backend serves as the primary stepping stone for compiling RPL with RPL.

---

## 2. Phased Roadmap

### Phase 0: Compiler Frontend Verification (✅ Complete)
- Strongly-typed Abstract Syntax Tree ([`crates/rpl_ast`](../crates/rpl_ast)).
- Logos-based lexical tokenizer with newline semantics and string interpolation ([`crates/rpl_lexer`](../crates/rpl_lexer)).
- Recursive Descent + Pratt parser for expressions, precedence climbing, and pipe operator `|>` ([`crates/rpl_parser`](../crates/rpl_parser)).
- Semantic type checker with Kleene 3-state ternary logic verification and move semantics ([`crates/rpl_typechecker`](../crates/rpl_typechecker)).
- Comprehensive test suite (47 tests passing, 0 clippy warnings).

---

### Phase 1: Portable C99 Backend & Cross-Platform CLI (✅ Complete)
* **Goal:** Full cross-platform code generation targeting standard C99, executable immediately on Windows and Linux.
* **Key Components:**
  * **[`crates/rpl_codegen_c`](../crates/rpl_codegen_c):**
    - C99 code generator translating validated AST into clean, readable C.
    - Representation of `Trit` as an 8-bit signed enum (`-1` = false, `0` = unknown, `1` = true).
    - Basic primitives (`Int`, `Float`, `Bool`, `String`), functions, control flow (`if`, `while`, `for`, `match`).
    - String interpolation lowering and memory management for scoped heap buffers.
    - Minimal, header-only runtime library (`rpl_runtime.h`) for string manipulation and collections.
  * **[`crates/rpl_cli`](../crates/rpl_cli):**
    - `rpl run <file.rpl>`: Generates C99, detects host C compiler (`clang`, `gcc`, or `cl.exe`), compiles, and executes.
    - `rpl build <file.rpl> -o <binary>`: Generates an optimized standalone `.exe` (Windows) or ELF executable (Linux).
    - `rpl check <file.rpl>`: Runs fast frontend validation (lexing, parsing, typechecking) without code generation.

---

### Phase 2: In-Memory Cranelift JIT Engine (`crates/rpl_codegen_cranelift`) (✅ Complete — v0.2 "Tohtlane")
* **Status:** Fully completed, tested, and integrated into `rpl_cli` as the default execution engine for `rpl run`.
* **Goal:** Zero external dependencies for interactive execution and lightning-fast developer iteration.
* **Key Components & Verified Deliverables:**
  * **[`crates/rpl_codegen_cranelift`](../crates/rpl_codegen_cranelift):**
    - High-performance in-memory JIT compiler utilizing `cranelift-jit` and `cranelift-module`.
    - Direct compilation of arithmetic, ternary logic (`Trit`), variables, comparisons, loops (`while`), conditionals (`if`/`else`), and pattern matching (`match`) directly to native host machine code (x86_64, AArch64).
    - Host-native calling convention management (`WindowsFastcall` on Windows, `SystemV` on Linux).
    - FFI bridge connecting Cranelift machine code to RPL runtime helper functions (`rpl_print_str`, `rpl_print_trit`, `rpl_print_i64`).
    - Sub-millisecond execution lifecycle directly from AST without writing temporary disk files.
  * **CLI Integration (`crates/rpl_cli`):**
    - `rpl run <file.rpl>`: Executes immediately via in-memory Cranelift JIT by default.
    - `rpl run --via-c <file.rpl>`: Explicit opt-in to portable C99 compilation pipeline.
    - `rpl check <file.rpl>`: Runs fast frontend validation with ternary status indicators (`[+ + ?]`).
    - Standardized version identification (`rpl 0.2+3 "Tohtlane"` with host target triple and active backends).
* **Test Verification:**
  - Complete workspace test suite passing (73 unit/integration tests).
  - Clean Clippy analysis across all crates.
  - End-to-end integration tests in `crates/rpl_cli/tests/cli_tests.rs` and `crates/rpl_codegen_cranelift/tests/jit_tests.rs`.

---

### Phase 3: Standard Library & Concurrency Runtime
* **Goal:** Realize RPL's promise of structured concurrency and expressive standard types.
* **Key Components:**
  * Runtime scheduler for `spawn:` blocks (lightweight task dispatch / thread pool).
  * Lock-free typed communication channels (`Channel[T]`).
  * `parallel for` work-stealing distribution across CPU cores.
  * Standard library modules: `math`, `io`, `fs`, `net`, `time`.

---

### Phase 4: Self-Hosting Compiler (`rpl-in-rpl`)
* **Goal:** The ultimate milestone of language maturity — compiling RPL using a compiler written entirely in RPL.
* **Key Milestones:**
  1. Port `rpl_lexer` and `rpl_ast` to `.rpl` source files.
  2. Implement `rpl_parser` and `rpl_typechecker` in idiomatic RPL.
  3. Compile the RPL-written compiler using the Phase 1 C99 backend (Stage 1 bootstrap).
  4. Verify the binary can compile its own source code identically (Stage 2 bootstrap).
  5. The compiler becomes fully self-sufficient and detached from the initial Rust implementation.

---

## 3. Guiding Invariants Across All Phases

1. **Scoping Rules:** Scopes always open with `:` and terminate exclusively with `end`. No `{}` or `;` may ever be emitted or accepted.
2. **Indentation Freedom:** Whitespace is non-semantic. Code generation and formatting must never rely on rigid indentation rules.
3. **Deterministic Memory Release:** Avoid global garbage collection. Stick to RAII, deterministic lexical scope drops, and ownership transfer.
4. **Cross-Platform Parity:** Every stage must maintain 100% feature and test parity across Windows and Linux.
