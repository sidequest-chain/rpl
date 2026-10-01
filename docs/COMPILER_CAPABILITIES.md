# RPL Compiler Capabilities & Implementation Status

> **Target Version:** 0.2+3 "Tohtlane"  
> **Purpose:** Authoritative technical snapshot of currently implemented, working compiler features versus roadmap items. Designed specifically for LLM agents, compiler developers, and architectural discussions to propose well-scoped language extensions without drifting from the active codebase.

---

## 1. Executive Summary

RPL (Running Pseudo Language) is a compiled, zero-GC systems language designed to execute pseudocode with bare-metal speed. As of version `0.2+3`, the compiler features a dual-backend architecture:
1. **Phase 1: C99 Transpiler (`rpl_codegen_c`)** — Generates human-readable, self-contained C99 compiled via GCC/Clang/MSVC.
2. **Phase 2: In-Memory Cranelift JIT (`rpl_codegen_cranelift`)** — In-memory machine code compilation executing `.rpl` files directly with sub-10ms latency via `rpl run`.
3. **Developer Tooling (`rpl_lsp`, `rpl_cli`)** — Built-in CLI commands (`run`, `build`, `check`, `lsp`) and zero-dependency LSP supporting semantic tokens, live hover documentation, and syntax/type diagnostics for VS Code and Zed.

---

## 2. Implemented Language Features (100% Working)

The following language constructs are fully implemented across the parser, typechecker, and at least one execution backend:

### 2.1. Structural Scoping & Grammar
- **Invariant Scoping:** Scopes open exclusively with `:` and close exclusively with `Token::End` (`end`, or labeled ends like `end fn`, `end for`, `end match`, `end type`).
- **Punctuation-Free:** No structural curly braces `{}` and no statement semicolons `;`. Statements terminate on `\n`.
- **Indentation Freedom:** Whitespace is not semantically significant for scoping (unlike Python). Indentation does not trigger parser errors.

### 2.2. Data Types & Literals
- **Scalar Types:**
  - `Int`: 64-bit signed integer (`i64` in Rust, `int64_t` in C99, `I64` in Cranelift).
  - `Float`: 64-bit IEEE 754 floating-point (`f64` in Rust, `double` in C99).
  - `Bool`: Standard boolean (`true`, `false`).
  - `String`: UTF-8 string literals with interpolation syntax `$variable` and `$(expr)`.
- **First-Class Ternary Logic (`Trit`):**
  - Native 3-valued Kleene logic type with literals `true`, `false`, `unknown`.
  - Truth tables enforced in compiler and codegen:
    - `and`: `unknown and false => false`, `unknown and true => unknown`
    - `or`: `unknown or true => true`, `unknown or false => unknown`
    - `not`: `not unknown => unknown`, `not false => true`
  - Compile-time exhaustiveness checking rejecting missing `Trit` match cases.
- **Composite Structs (`TypeDecl`):**
  - Struct definition: `type Point: x: Int, y: Int end`
  - Struct block initialization:
    ```rpl
    let p = Point:
        x: 10
        y: 20
    end
    ```
  - Member access: `p.x`, `p.y`.

### 2.3. Variables, Mutability & Ownership
- **Immutable Bindings:** `let name: Type = expr` (default immutable, compile error on reassignment).
- **Mutable Bindings:** `let mut name: Type = expr` (permits reassignment `name = new_expr`).
- **Type Inference:** Explicit annotations are optional (`let x = 42` infers `Int`).
- **Affine / Move Semantics:** Typechecker tracks value moves and statically rejects use-after-move.

### 2.4. Control Flow & Pattern Matching
- **Conditional Branching:** `if condition: ... else: ... end`.
- **Sequential Iteration:** `for item in start..end: ... end` (supports inclusive/exclusive ranges).
- **Pattern Matching (`match`):**
  - Literal patterns (`case 42:`, `case true:`, `case unknown:`).
  - Wildcard pattern (`case _:`).
  - Pattern guards (`case x if x > 10:`).

### 2.5. Functions, Lambdas & Pipeline
- **Named Functions:** `fn name(param: Type) -> ReturnType: ... end`.
- **Pipe Operator (`|>`):** Passes left-hand expression as argument to the right-hand function (`x |> double |> print`).
- **Lambdas / Closures:** Anonymous inline functions `(x, y) => x + y`.

---

## 3. Compiler Backend Capability Matrix

| Feature / AST Node | Parser & AST | Typechecker | C99 Backend (`rpl build`) | Cranelift JIT (`rpl run`) | LSP Support |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Arithmetic (`+`, `-`, `*`, `/`, `%`)** | ✅ Full | ✅ Full | ✅ Full (`int64_t`, `double`) | ✅ Full (`i64`) | ✅ Full |
| **Relational (`==`, `!=`, `<`, `<=`, `>`, `>=`)** | ✅ Full | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **Logical (`and`, `or`, `not`)** | ✅ Full | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **Trit Ternary Logic** | ✅ Full | ✅ Exhaustive | ✅ Full (`rpl_trit_t`) | ✅ Full (`i8` / Kleene) | ✅ Diagnostics |
| **Mutable Variables (`let mut`)** | ✅ Full | ✅ Enforced | ✅ Full (`int64_t b = ...`) | ✅ Full (Cranelift Var) | ✅ Tokens |
| **Use-After-Move Check** | ✅ Full | ✅ Rejection | N/A (Static check) | N/A (Static check) | ✅ Diagnostics |
| **Range For Loops (`1..5`)** | ✅ Full | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **Struct Declaration & Access** | ✅ Full | ✅ Type verified | ✅ Full (`typedef struct`) | ✅ Full (Offset read) | ✅ Hover & Tokens |
| **Struct Block Init (`Point: ... end`)** | ✅ Full | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **Pipe Operator (`\|>`)** | ✅ Full | ✅ Full | ✅ Full | ⚠️ Via function call | ✅ Tokens |
| **String Interpolation (`$var`)** | ✅ Full | ✅ Full | ✅ Full (`rpl_str_concat`) | ⚠️ Plain string only | ✅ Tokens |
| **Match Statement** | ✅ Full | ✅ Exhaustive | ✅ Full (`switch` & `if-else`) | ⚠️ Simple literal | ✅ Exhaustive Err |
| **Built-in `print` / `println`** | ✅ Full | ✅ Full | ✅ Full (`printf`) | ✅ Full (`stdout`) | ✅ Hover |
| **List Literals (`[1, 2, 3]`)** | ✅ Parsed | ⚠️ In progress | ⚠️ Prototype | ⚠️ Not wired | ✅ Tokens |
| **Index Access (`arr[i]`)** | ✅ Parsed | ⚠️ In progress | ⚠️ Prototype | ⚠️ Not wired | ✅ Tokens |
| **`spawn: ... end`** | ✅ Parsed | ⚠️ AST only | ❌ Phase 3 "Kratt" | ❌ Phase 3 "Kratt" | ✅ Tokens |
| **`channel: ... end`** | ✅ Parsed | ⚠️ AST only | ❌ Phase 3 "Kratt" | ❌ Phase 3 "Kratt" | ✅ Tokens |
| **`parallel for`** | ✅ Parsed | ⚠️ AST only | ❌ Phase 3 "Kratt" | ❌ Phase 3 "Kratt" | ✅ Tokens |

---

## 4. Current Architecture & Crates Map

```text
crates/
├── rpl_ast/               # Pure AST data structures, Span coordinates, Kleene Trit logic
├── rpl_lexer/             # Logos-based zero-copy lexer, string interpolation parser
├── rpl_parser/            # Recursive descent statements + Pratt expression parsing
├── rpl_typechecker/       # Semantic analysis, type inference, Trit exhaustiveness, move checking
├── rpl_codegen_c/         # Standalone C99 transpiler emitting clean, compilable C
├── rpl_codegen_cranelift/ # Fast in-memory JIT code generation via Cranelift JITModule
├── rpl_lsp/               # Zero-dependency Language Server Protocol implementation
└── rpl_cli/               # Unified CLI binary (run, build, check, lsp)
```

---

## 5. High-Value Extension Opportunities (Prompts for Partner LLM)

When brainstorming or requesting proposals from a partner AI model, focus on these concrete, architecturally ready areas:

### Opportunity A: Standard Library Built-ins & Intrinsics
- **Current State:** Only `print` and `println` exist as intrinsics in C99 and Cranelift runtime bridges.
- **Discussion Prompts for Partner:**
  - What minimal set of math built-ins (`abs`, `min`, `max`, `sqrt`, `pow`) should be wired directly into the compiler?
  - How should file I/O (`read_file(path) -> String`, `write_file(path, content)`) be bridged cleanly in C99 and Cranelift without adding external C dependencies?

### Opportunity B: Array / Slice Collection Mechanics
- **Current State:** `Expr::List` and `Expr::Index` exist in AST and parser, but have not been hooked to a deterministic heap/stack layout in `rpl_codegen_c` or Cranelift.
- **Discussion Prompts for Partner:**
  - Should arrays be fixed-size stack arrays (`[Int; 4]`), dynamically resizable slices (`List<Int>`), or both?
  - How to maintain zero-GC deterministic memory cleanup for allocated lists when they exit lexical scope?

### Opportunity C: Result / Option Types vs. Trit-Driven Error Handling
- **Current State:** RPL strictly forbids `null` and exceptions. Trit ternary logic (`true`, `false`, `unknown`) is native.
- **Discussion Prompts for Partner:**
  - Can `unknown` be used idiomaticly for missing or uninitialized values, or should RPL introduce a native algebraic `Option` / `Result` enum?
  - What syntax fits the `:` ... `end` philosophy best for error propagation (e.g. `try`, `?`, or pattern matching)?

### Opportunity D: Phase 3 "Kratt" Concurrency Design
- **Current State:** `spawn`, `channel`, and `parallel for` keywords exist in the grammar.
- **Discussion Prompts for Partner:**
  - What lightweight C99 threading backend (e.g. `tinycthread` or pthreads/Win32 threads wrapper) best fits the zero-GC shared-nothing channel architecture?
  - How should channels integrate with `match` or select statements in pseudocode syntax?
