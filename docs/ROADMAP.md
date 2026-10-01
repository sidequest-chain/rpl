# RPL — STRATEGIC ARCHITECTURE ROADMAP & EVOLUTION MATRIX (ROADMAP.md)
Document ID: RPL-ROAD-2026-V2  
Classification: PUBLIC OPEN SOURCE / EUPL-1.2  
Status: ACTIVE ROADMAP  
Author: RPL Core Compiler Team  
Active Release: 0.2+3 "Tohtlane"  

---

## 1. ARCHITECTURAL CONTEXT & HYBRID EXECUTION STRATEGY

**Running Pseudo Language (RPL)** is a modern, high-performance programming language designed to look and feel like clean, natural pseudocode while executing with bare-metal C and machine-code velocity.

### 1.1. The Hybrid Execution Model
To balance rapid cross-platform deployment (Windows & Linux) with instantaneous developer feedback and native execution speed, RPL enforces a dual-engine architecture:

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

1. **Immediate Cross-Platform Reliability (Windows + Linux):**
   * Emitting high-performance, standard C99 abstracts away OS differences, system ABIs, and platform-specific linkers.
   * Complex language capabilities (string interpolation, dynamic arrays, Kleene ternary logic, and deterministic scope cleanup) are delivered cleanly and portably using C99 and standard libc.
2. **Instant Developer Feedback:**
   * In-memory Cranelift JIT eliminates the latency of spawning external C compilers during daily development, providing sub-millisecond in-memory compilation for `rpl run`, interactive experimentation, and automated testing.
3. **The Bootstrap Stepping Stone for Self-Hosting:**
   * The C99 backend serves as the primary stepping stone for compiling RPL with RPL (`rpl-in-rpl`), mirroring the proven pathways of C++, Nim, and early Rust.

---

## 2. PRIORITY-TIERED ROADMAP MATRIX

| Domain / Capability | Priority | Target Subsystem | Status | Primary Objective |
| :--- | :--- | :--- | :--- | :--- |
| **Compiler Frontend & Grammar Verification** | **P0 (Base)** | `lexer / parser / typechecker` | **Completed** | Full AST, Logos tokenizer with newline semantics, Pratt parser, Kleene 3-state trit logic. |
| **Portable C99 Transpiler Backend** | **P0 (Base)** | `codegen_c / cli` | **Completed** | Clean C99 emission, host compiler detection (`clang`/`gcc`/`cl.exe`), standalone binary generation (`rpl build`). |
| **In-Memory Cranelift JIT Engine** | **P0 (Base)** | `codegen_cranelift / cli` | **Completed** | Sub-ms native code execution for `rpl run`, WindowsFastcall/SystemV ABIs, zero-dependency CLI runtime (v0.2 "Tohtlane"). |
| **Zero-Dependency Language Server Protocol (LSP)** | **P1 (High)** | `rpl_lsp / cli` | **Completed** | Embedded `rpl lsp` engine with diagnostics, hover documentation, and LSP 3.17 semantic tokens. |
| **Local Editor Integration (Variant A: File Association)** | **P1 (High)** | `editors / zed / vscode` | **Immediate Focus** | Rapid zero-build Zed and VS Code integration via user file associations and LSP semantic token mapping. |
| **Standard Library: Core IO, Math & String Collections** | **P1 (High)** | `stdlib / runtime` | **Next Up** | First-class string utilities, file system access (`fs`), console I/O, math primitives, and dynamic collections. |
| **Structured Concurrency Runtime (`spawn:` & Channels)** | **P2 (High)** | `runtime / codegen` | **Medium-Term** | Lightweight M:N cooperative task scheduler, lock-free typed channels (`Channel[T]`), and channel select semantics. |
| **Multi-Core Data Parallelism (`parallel for`)** | **P2 (High)** | `typechecker / codegen` | **Medium-Term** | Safe work-stealing thread pool distribution across CPU cores for loop ranges and batch data processing. |
| **Full Native Zed Extension (Variant B: Tree-sitter & Wasm Component)** | **P3 (Planned)** | `editors/zed` | **Scheduled** | Official standalone Zed extension package featuring native Tree-sitter C grammar parser and `wasm32-wasip2` Component Model. |
| **Exhaustive Borrow & Ownership Verification ("Siil")** | **P3 (Vision)** | `rpl_typechecker` | **Strategic Vision** | Compile-time affine type system preventing data races, use-after-free, and concurrent mutation without garbage collection. |
| **Self-Hosting Compiler Milestone (`rpl-in-rpl`)** | **P3 (Vision)** | `compiler (all)` | **Milestone 1.0** | Compiling the complete RPL compiler toolchain using RPL itself ("Põhja Konn"). |
| **External Standalone LSP Daemon Binary** | **P4 (Parked)** | `rpl_lsp` | **Deferred** | Separating `rpl_lsp` into a detached binary; deferred because embedding inside `rpl lsp` eliminates installation friction. |
| **Heavy Tree-sitter CLI Toolchain Build Requirement** | **P4 (Parked)** | `tooling` | **Deferred** | Requiring node/npm/gyp to build grammar locally; parked until official upstream tree-sitter integration is standardized. |

---

## 3. DETAILED HORIZON SPECIFICATIONS

### 3.1. Completed Milestones (Foundational Generations)

1. **Phase 0: Compiler Frontend Verification (v0.1 Base)**
   - Strongly-typed Abstract Syntax Tree ([`crates/rpl_ast`](../crates/rpl_ast)).
   - Logos-based lexical tokenizer with newline significance outside parentheses and interpolation detection ([`crates/rpl_lexer`](../crates/rpl_lexer)).
   - Recursive Descent + Pratt parser for precedence climbing, binary/unary expressions, and pipe operator `|>` ([`crates/rpl_parser`](../crates/rpl_parser)).
   - Semantic type checker enforcing Kleene 3-state ternary logic truth tables and non-exhaustive `match` rejection ([`crates/rpl_typechecker`](../crates/rpl_typechecker)).
   - 100% test coverage across frontends with zero compiler warnings.

2. **Phase 1: Portable C99 Transpiler & Native CLI (v0.1 "Puulane")**
   - C99 code generator translating AST into clean, human-readable C ([`crates/rpl_codegen_c`](../crates/rpl_codegen_c)).
   - Representation of `Trit` as an 8-bit signed enum (`-1` = false, `0` = unknown, `1` = true).
   - Minimal header-only runtime (`rpl_runtime.h`) handling string interpolation and scoped heap buffers.
   - Cross-platform CLI driver (`rpl run`, `rpl build`, `rpl check`) with automatic host C compiler detection (`clang`, `gcc`, `cl.exe`).

3. **Phase 2: In-Memory Cranelift JIT Engine (v0.2 "Tohtlane")**
   - High-performance in-memory JIT backend powered by `cranelift-jit` and `cranelift-module` ([`crates/rpl_codegen_cranelift`](../crates/rpl_codegen_cranelift)).
   - Direct machine code lowering for arithmetic, Kleene ternary logic, structs, comparisons, loops, and control flow.
   - Host ABI calling convention negotiation (`WindowsFastcall` on Windows, `SystemV` on Linux/macOS).
   - Sub-millisecond execution lifecycle directly from AST memory with zero intermediate disk artifacts.
   - Embedded Language Server Protocol daemon (`rpl lsp`) with diagnostics, hover inspection, and LSP 3.17 semantic tokens.

---

### 3.2. P1: Immediate Focus (Active Next Steps)

1. **Instant Editor Developer Experience (Variant A: Local File Association):**
   - Provide turnkey `.zed/settings.json` and VS Code configurations associating `.rpl` files with existing native grammars and mapping LSP semantic tokens directly.
   - Enables immediate syntax highlighting, hover documentation, and compile-on-save diagnostics without requiring external packaging or WebAssembly toolchains.
   - Documented in [`docs/IDE_SETUP.md`](../docs/IDE_SETUP.md).

2. **Standard Library Core Primitives (`math`, `io`, `fs`, `strings`):**
   - Native modules for filesystem reading/writing, terminal formatted I/O, string manipulation (split, join, replace, regex), and mathematical operations.
   - Runtime memory safety guarantees ensuring buffers are automatically released at scope exit (`end`).

---

### 3.3. P2: Medium-Term Horizon (Concurrency & Scaling)

1. **Structured Concurrency Runtime (`spawn:` Blocks):**
   - Lightweight cooperative task scheduler dispatching `spawn:` jobs across worker threads.
   - Deterministic scope termination: parent blocks await child task completion before closing scope unless explicitly detached.

2. **Lock-Free Communication Channels (`Channel[T]`):**
   - Bounded and unbounded typed message channels facilitating safe inter-task communication.
   - Select statement integration allowing tasks to await multiple channels simultaneously.

3. **Multi-Core Data Parallelism (`parallel for`):**
   - Automatic work-stealing chunking for loop iterations across physical CPU cores.
   - Compile-time immutability validation preventing data races during parallel iterations.

---

### 3.4. P3: Strategic Vision (Language Independence & Official Packaging)

1. **Variant B: Full Native Zed Extension Package (`editors/zed`):**
   - Build a formal standalone Zed extension implementing the Wasm Component Model (`wasm32-wasip2`).
   - Package a dedicated Tree-sitter C grammar parser (`tree-sitter-rpl`) to deliver native structural folding, code outlines, and rainbow bracket colorization.
   - Automated local developer extension installer (`zed: install dev extension`) with zero external network dependencies.

2. **Exhaustive Borrow & Ownership Verification ("Siil" Milestone):**
   - Compile-time affine type system preventing data races, use-after-free, and concurrent mutation without a tracing garbage collector.

3. **Self-Hosting Compiler Milestone (`1.0 "Põhja Konn"`):**
   - Re-implementing `rpl_lexer`, `rpl_parser`, `rpl_typechecker`, and `rpl_codegen_c` in idiomatic RPL source files.
   - Two-stage bootstrap verification: compiling RPL with the Rust-based C99 backend, then verifying self-compilation identity.

---

### 3.5. P4: Deferred Technical Parking Lot

| Parked Initiative | Why Deferred (The Why)? | Unblocking Trigger / Prerequisite |
| :--- | :--- | :--- |
| **Variant B Full Tree-sitter Packaging for Local Use** | Requiring Tree-sitter compilation and `wasm32-wasip2` WebAssembly component builds creates unnecessary friction and compiler stalls during local language experimentation. Variant A delivers 100% of semantic tokens and LSP diagnostics in seconds. | When the language syntax stabilizes, public Zed extension publishing is initiated, or `wasm32-wasip2` toolchains are ubiquitous on all developer workstations. |
| **Detached Standalone `rpl-lsp` Binary** | A unified single-binary distribution (`rpl lsp`) eliminates path lookup issues, package desynchronization, and simplifies installer scripts (`install.ps1`). | If embedded LSP dependencies inflate binary size beyond acceptable single-binary targets. |
| **Heavy External NPM/Node Grammar Builders** | Requiring Node.js, `node-gyp`, and Python just to parse RPL files in editors violates RPL's zero-dependency philosophy. | When an official upstream Tree-sitter repository is created and precompiled into static C sources. |
| **Global Garbage Collection Runtime** | A tracing GC introduces non-deterministic stop-the-world pauses, memory bloat, and complicates embedded execution targets. | Rejected permanently in favor of deterministic scope-based RAII and affine ownership verification. |

---

## 4. COMPILER INVARIANTS & OPERATIONAL BOUNDARIES

1. **Block Delimiters:** Scopes always open with `:` and close exclusively with `end`. Structural curly braces `{}` and semicolons `;` must **never** be parsed or emitted under any circumstances.
2. **Whitespace Flexibility:** Whitespace and indentation carry no semantic scoping significance. Structural blocks are bound purely by `:` and `end`.
3. **Deterministic Memory Release:** No global tracing garbage collector. All heap allocations are managed via deterministic lexical scope drops, RAII, and ownership transfer.
4. **Cross-Platform Parity:** Every stage and feature must maintain 100% test and execution parity across Windows and Linux.
5. **Trinary Diagnostic Verification:** Trit values (`true`, `false`, `unknown`) are first-class language citizens and must enforce exhaustive pattern matching at compile time.

---

## 5. ROADMAP LIFECYCLE & MAINTENANCE PROTOCOL

1. **Active Work Selection:** Development initiatives are selected strictly from the top of the **P1 (Immediate Focus)** tier without premature version locking.
2. **Version Synchronization:** When milestones complete, the monotonic patch or minor milestone codename is bumped in `VERSION`, `Cargo.toml`, and registered in `AGENTS.md`.
3. **Documenting Architectural Changes:** Architectural decisions and rejected alternatives must follow the Tier 2 Full HIDC commit protocol and be synchronized across documentation.
4. **Parking Protocol:** Unscheduled ideas or heavy toolchain requirements that arise during development are immediately cataloged in Section 3.5 (Deferred Parking Lot) with clear "Why Deferred" and "Unblocking Trigger" criteria.
