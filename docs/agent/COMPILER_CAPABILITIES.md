# RPL Compiler Capabilities & Implementation Status (docs/agent/COMPILER_CAPABILITIES.md)

> **Document Role:** Compiler Ground Truth & Active Capability Matrix  
> **Target Audience:** Autonomous AI Agents & Compiler Developers (**Ground Truth Authority**)  
> **Active Milestone:** 0.2 "Tohtlane" (Active patch: see [VERSION](../../VERSION))  
> **Authority Level:** **Absolute SSoT** for what language constructs 100% compile, run, and pass test suites today.  
> **Boundary Notice:** For long-term syntax vision and planned features (e.g. channels, `Result`, arrays), consult [docs/spec/PROJECT_SPEC.md](../spec/PROJECT_SPEC.md). For end-user tutorials, consult [docs/user/LANGUAGE_GUIDE.md](../user/LANGUAGE_GUIDE.md).

---

## 1. Executive Summary

RPL (Running Pseudo Language) is a compiled, zero-GC systems language designed to execute pseudocode with bare-metal speed. As of milestone `0.2 "Tohtlane"`, the compiler features a dual-backend architecture:
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
- **Conditional Branching:** `if condition: ... else if: ... else: ... end`.
- **Sequential Iteration:** `for item in start..end: ... end` (supports inclusive/exclusive ranges).
  > [!WARNING]
  > **`while` loops are NOT implemented in milestone 0.2.** The `while` keyword is recognized by the lexer, but neither the AST (`rpl_ast::Stmt`), parser (`rpl_parser::stmt`), nor codegen backends support `while`. Use `for i in start..end:` with `if`/`match` state flags for loops in 0.2.
- **Pattern Matching (`match`):**
  - Literal patterns (`case 42:`, `case true:`, `case unknown:`).
  - Wildcard pattern (`case _:`).
  - Pattern guards (`case x if x > 10:`).

### 2.5. Functions, Lambdas & Pipeline
- **Named Functions (`fn`):**
  - Full syntax: `fn name(param: Type, ...) -> ReturnType: ... end fn` (or plain `end`).
  - Parameter type annotations are **strictly mandatory** (e.g. `fn double_val(n: Int) -> Int:`).
  - Return type is optional (omitting `-> ReturnType` defaults to unit/void).
  - Must be declared at top level. All callable code must be structured as `fn` declarations.
- **Pipe Operator (`|>`):** Passes left-hand expression as argument to the right-hand function (`x |> double |> print` or `x |> double()`).
- **Lambdas / Inline Closures (`(a, b) => expr`):**
  > [!WARNING]
  > **Lambdas are currently untyped inline AST expressions only.**
  > - Parameters must be bare identifiers without type annotations: `x => x + 1` or `(x, y) => x + y`. Syntax like `(x: Int) => ...` is rejected by the parser.
  > - **First-class callable variable bindings (`let f = ...; f()`) are NOT supported in milestone 0.2.** Functions cannot be stored in variables and invoked as `f(...)`. Always use top-level `fn name(...)` declarations for callable logic.

### 2.6. Two-Tier Input / Output System (Convenience & Streams)
RPL features a native, zero-dependency two-tier I/O architecture combining high-level pseudocode convenience with long-lived system streaming:
- **Layer 1: Zero-Ceremony Pseudocode Convenience (Atomic)**
  - `input() -> String`: Reads a line from `stdin` with trailing `\r`/`\n` stripped.
  - `read_file(path: String) -> String`: Reads the entire file into memory and immediately closes it. Gracefully returns `""` on I/O error.
  - `write_file(path: String, content: String) -> Bool`: Overwrites or creates the file with content and immediately closes it. Returns `true` on success.
  - `append_file(path: String, content: String) -> Bool`: Appends content to the end of the file and immediately closes it. Returns `true` on success.
  - Seamless pipeline composition: `"telemetry.log" |> read_file |> println`.
- **Layer 2: Long-Lived System Streams & Handles (Daemons / Servers)**
  - `File`: First-class opaque handle type representing an active system stream (`FILE*` in C99, `*mut JitFile` in Cranelift JIT).
  - `open_file(path: String, mode: String) -> File`: Opens a file stream with mode `"r"`, `"w"`, or `"a"`.
  - `read_line(file: File) -> String`: Reads the next line from the open file handle without closing it.
  - `write_line(file: File, line: String) -> Bool`: Writes a line of text followed by newline to the open stream with automatic flush (`fflush`), keeping the file open.
  - `close_file(file: File)`: Safely flushes and closes the stream handle.
- **Affine Ownership & Static Use-After-Close Protection:**
  - `close_file(f)` and `f |> close_file` take ownership of `f` (move semantics).
  - Any subsequent attempt to read, write, or re-close `f` is rejected at compile time by the typechecker with `Use of moved value 'f'`, preventing dangling handle vulnerabilities.

> [!IMPORTANT]
> **Strict Code Generation Directives for AI Agents & Developers:**
> 1. **Functions:** Always declare callable logic using `fn name(param: Type) -> RetType: ... end fn`. Never use `let f = ... => ...` with the intention of calling `f()`.
> 2. **Parameter Types:** Function parameters require explicit uppercase types (`Int`, `Float`, `Bool`, `String`, `Trit`, `File`, or a struct name). Lowercase types like `int` are rejected.
> 3. **Block Delimiters:** Scopes always open with `:` and close with `end` or labeled `end <keyword>` (`end fn`, `end for`, `end match`, `end type`). Never use curly braces `{}` or semicolons `;`.
> 4. **Print & Interpolation:** `println("Count: $counter, Sensor: $s.id")` works out-of-the-box in both C99 and Cranelift JIT.
> 5. **File I/O:** Use `read_file` / `write_file` for quick scripts and `open_file` / `write_line` / `close_file` for streaming loggers and daemons. Always ensure `close_file` is called once per handle.

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
| **`while` Loops (`while cond:`)** | ⚠️ Lexer only | ❌ Not in AST | ❌ Not in parser | ❌ Not supported | ❌ None |
| **Struct Declaration & Access** | ✅ Full | ✅ Type verified | ✅ Full (`typedef struct`) | ✅ Full (Offset read) | ✅ Hover & Tokens |
| **Struct Block Init (`Point: ... end`)** | ✅ Full | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **Named Functions (`fn name(...)`)** | ✅ Full | ✅ Full | ✅ Full (`rpl_fn`) | ✅ Full | ✅ Full |
| **Pipe Operator (`\|>`)** | ✅ Full | ✅ Full | ✅ Full | ✅ Full | ✅ Tokens |
| **String Interpolation (`$var`)** | ✅ Full | ✅ Full | ✅ Full (`rpl_str_concat`) | ✅ Full (`rpl_jit_str_concat`) | ✅ Tokens |
| **Match Statement** | ✅ Full | ✅ Exhaustive | ✅ Full (`switch` & `if-else`) | ⚠️ Simple literal & Trit | ✅ Exhaustive Err |
| **Built-in `print` / `println`** | ✅ Full | ✅ Full | ✅ Full (`printf`) | ✅ Full (`stdout`) | ✅ Hover |
| **Console Input (`input()`)** | ✅ Full | ✅ Full (`() -> String`) | ✅ Full (`rpl_input`) | ✅ Full (`rpl_jit_input`) | ✅ Hover |
| **Convenience File I/O (`read_file`, `write_file`, `append_file`)** | ✅ Full | ✅ Full | ✅ Full (`rpl_*`) | ✅ Full (`rpl_jit_*`) | ✅ Hover |
| **System Stream Handle (`File`)** | ✅ Full | ✅ Full (Opaque) | ✅ Full (`rpl_file_t`) | ✅ Full (`*mut JitFile`) | ✅ Hover & Tokens |
| **Stream File Operations (`open_file`, `read_line`, `write_line`)** | ✅ Full | ✅ Full | ✅ Full (`fopen`/`fgets`/`fputs`) | ✅ Full (`rpl_jit_*`) | ✅ Hover |
| **Affine Stream Closing (`close_file`)** | ✅ Full | ✅ Enforced Move | ✅ Full (`fclose`) | ✅ Full (`rpl_jit_close_file`) | ✅ Hover & Move Err |
| **`assert(cond: Bool)`** | ❌ Not in lexer/parser | ❌ Not in `env.rs` | ❌ Not in runtime | ❌ Not supported | ❌ None |
| **`len(val: String)`** | ❌ Not in lexer/parser | ❌ Not in `env.rs` | ❌ Not in runtime | ⚠️ Declared symbol only (not wired) | ❌ None |
| **Lambdas / Closures (`=>`)** | ✅ Parsed | ⚠️ Untyped expr only | ⚠️ Prototype | ⚠️ Prototype | ✅ Tokens |
| **Callable Variables (`let f = ...; f()`)** | ❌ Not supported | ❌ Not supported | ❌ Not supported | ❌ Not supported | ❌ TypeError |
| **List Literals (`[1, 2, 3]`)** | ✅ Parsed | ⚠️ In progress | ⚠️ Prototype | ⚠️ Not wired | ✅ Tokens |
| **Index Access (`arr[i]`)** | ✅ Parsed | ⚠️ In progress | ⚠️ Prototype | ⚠️ Not wired | ✅ Tokens |
| **`Result[T, E]` / `Ok` / `Error`** | ✅ Parsed | ⚠️ Permissive stub | ❌ Not supported | ❌ Undefined Symbol | ✅ Tokens |
| **`Map[K, V]`** | ✅ Parsed | ⚠️ Permissive stub | ❌ Not supported | ❌ Not supported | ✅ Tokens |
| **`spawn: ... end`** | ✅ Parsed | ⚠️ AST only | ❌ Phase 3 "Kratt" | ❌ Phase 3 "Kratt" | ✅ Tokens |
| **`channel: ... end`** | ✅ Parsed | ⚠️ AST only | ❌ Phase 3 "Kratt" | ❌ Phase 3 "Kratt" | ✅ Tokens |
| **`parallel for`** | ✅ Parsed | ⚠️ AST only | ❌ Phase 3 "Kratt" | ❌ Phase 3 "Kratt" | ✅ Tokens |

---

## 4. Current Architecture & Crates Map

```text
crates/
├── rpl_ast/               # Pure AST data structures, Span coordinates, Kleene Trit logic, Type::File
├── rpl_lexer/             # Logos-based zero-copy lexer, string interpolation parser
├── rpl_parser/            # Recursive descent statements + Pratt expression parsing
├── rpl_typechecker/       # Semantic analysis, type inference, Trit exhaustiveness, affine move checking
├── rpl_codegen_c/         # Standalone C99 transpiler with rpl_runtime.h (Two-Tier I/O)
├── rpl_codegen_cranelift/ # Fast in-memory JIT code generation and runtime ABI bridges
├── rpl_lsp/               # Zero-dependency Language Server Protocol (hover, semantic tokens, diagnostics)
└── rpl_cli/               # Unified CLI binary (run, build, check, lsp)
```

---

## 5. High-Value Extension Opportunities (Prompts for Partner LLM)

When brainstorming or requesting proposals from a partner AI model, focus on these concrete, architecturally ready areas:

### Opportunity A: Standard Library Built-ins & Intrinsics
- **Current State:** Zero-dependency I/O is fully implemented across C99 and Cranelift JIT (`input`, `read_file`, `write_file`, `append_file`, `open_file`, `read_line`, `write_line`, `close_file`).
- **Discussion Prompts for Partner:**
  - What minimal set of math built-ins (`abs`, `min`, `max`, `sqrt`, `pow`) should be wired directly into the compiler?
  - Should standard string manipulation built-ins (`length`, `starts_with`, `trim`) be introduced as intrinsics or standard methods?

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
