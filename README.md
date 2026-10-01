# RPL (Running Pseudo Language)

[![Milestone: 0.2 "Tohtlane"](https://img.shields.io/badge/milestone-0.2_%22Tohtlane%22-blue.svg)](docs/spec/ROADMAP.md)
[![Active Release: see VERSION](https://img.shields.io/badge/release-see%20VERSION-informational.svg)](VERSION)
[![License: EUPL 1.2](https://img.shields.io/badge/License-EUPL_1.2-blue.svg)](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
[![Build Status](https://img.shields.io/badge/tests-74%20passed-brightgreen.svg)]()
[![Language: Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)]()

> **Running Pseudo Language (RPL)** is a compiled, zero-garbage-collector systems programming language designed to eliminate the translation boundary between conceptual pseudocode and high-performance native execution.
>
> **Current Milestone:** `0.2 "Tohtlane"` (Active patch: see [`VERSION`](VERSION) / [`CHANGELOG.md`](CHANGELOG.md)). Target: host native with in-memory Cranelift JIT & C99 backends.

---

## Language Highlights

* **Executable Clarity:** Reads like clean algorithmic pseudocode. No curly braces `{}` and no statement semicolons `;`. Scopes open with `:` and close exclusively with `end`.
* **Indentation-Agnostic (Anti-Python):** Indentation has zero semantic significance. Scoping is strictly bounded by `:` and `end`, meaning 4 spaces, irregular tabs, or zero indentation parse identically.
* **Zero Garbage Collection:** Deterministic memory release bound to lexical scopes and compile-time ownership transfer (*move semantics*).
* **First-Class Ternary Logic (`Trit`):** Native language and type system support for Kleene 3-valued logic (`true`, `false`, `unknown`) with enforced pattern exhaustiveness.
* **Modern Syntax:** Clean pipe operator (`|>`), native string interpolation (`$var` and `$(expr)`), and expression-oriented design.

### RPL Code Sample

```rpl
fn evaluate_sensor(reading: Float, threshold: Float) -> Trit:
    if reading > threshold:
        return true
    else if reading < 0.0:
        return unknown
    else:
        return false
    end
end

fn main():
    let status: Trit = evaluate_sensor(-1.5, 10.0)
    
    let report = match status:
        case true: "Nominal"
        case false: "Below threshold"
        case unknown: "Telemetry uncertain, recalibrating"
    end

    println("System diagnostic: $report")
end
```

#### Indentation Freedom

Because block scoping is strictly bounded by `:` and `end`, indentation is purely stylistic:

```rpl
// Standard readable indentation:
fn calculate(x: Int) -> Int:
    let temp = x * 2
    return temp + 1
end

// Zero indentation (fully valid and parses identically):
fn calculate(x: Int) -> Int:
let temp = x * 2
return temp + 1
end
```

---

## Versioning Policy & Diagnostic Indicators

RPL enforces a strict release and diagnostic identity across all tooling and compilers:

### Version Format
```text
MAJOR.MINOR[+PATCH] "Codename"
```
* **Current Version:** `0.2+4 "Tohtlane"`
* **`MAJOR.MINOR`:** Architectural capability and subsystem milestone (`0.1` C99 transpiler, `0.2` in-memory Cranelift JIT). Base releases may appear without `+0`.
* **`+PATCH`:** Monotonic patch counter (`+1`, `+2`, ... `+x`) designating substantive fixes, maintenance updates, or refined iterations within the active milestone.
* **`"Codename"`:** Mythological creatures from Friedrich Reinhold Kreutzwald's Estonian folk heritage (*Eesti rahva ennemuistsed jutud*, 1866):
  - **Core Milestones:** `0.1 "Puulane"` → `0.2 "Tohtlane"` → `0.3 "Kratt"` → `0.4 "Tulihänd"` → `0.5 "Siil"` → `1.0 "Põhja Konn"` (self-hosting).
  - **Reserved Intermediate Milestones (if needed before 1.0):** `0.6 "Kodukäija"`, `0.7 "Murueit"`, `0.8 "Libahunt"`, `0.9 "Tark mees taskus"`.
* **Major Version Discipline:** The `0.x` series remains locked until self-hosting (`1.0 "Põhja Konn"`). Bumping `MAJOR` is strictly forbidden unless there is an unavoidable, fundamental paradigm shift in core language mechanics.
* **Package Schema Lock:** Libraries permanently lock their schema as either 3-part (`0.0.0`) or 4-part (`0.0.0.0`) upon initial publish.

### Trinary Diagnostic Feedback
In alignment with RPL's ternary logic (`true`, `false`, `unknown`), compiler stages report diagnostic vectors using Trits:
```text
[Syntax/Parser . Typechecker . Codegen]
```
* `[+ + +]`: Complete success (e.g., `rpl build`, `rpl run`).
* `[+ + ?]`: Check passed without codegen (e.g., `rpl check <file.rpl>`).
* `[+ - -]`: Typecheck failure (syntax valid, type analysis rejected).
* `[- - -]`: Lexer/Parser syntax failure (grammar error, compilation halted).

---

## Current Project Status

The project is structured as a modular Rust workspace and has completed both Phase 1 (portable C99 transpiler) and Phase 2 (in-memory Cranelift JIT compiler).

| Subsystem / Crate | Purpose | Status | Test Coverage |
| :--- | :--- | :---: | :---: |
| **[`crates/rpl_ast`](crates/rpl_ast)** | Strongly-typed AST, `Span` coordinates, Kleene `TritValue` | ✅ **Complete** | 9 tests |
| **[`crates/rpl_lexer`](crates/rpl_lexer)** | Logos tokenizer, 28 keywords, string interpolation, newlines | ✅ **Complete** | 9 tests |
| **[`crates/rpl_parser`](crates/rpl_parser)** | Recursive Descent (stmts) + Pratt parser (expressions) | ✅ **Complete** | 20 tests |
| **[`crates/rpl_typechecker`](crates/rpl_typechecker)** | Semantic typing, trit exhaustiveness, move semantics | ✅ **Complete** | 13 tests |
| **[`crates/rpl_codegen_c`](crates/rpl_codegen_c)** | High-performance C99 transpiler backend | ✅ **Complete (Phase 1)** | 7 tests |
| **[`crates/rpl_codegen_cranelift`](crates/rpl_codegen_cranelift)** | High-performance in-memory Cranelift JIT compiler | ✅ **Complete (Phase 2)** | 5 tests |
| **[`crates/rpl_cli`](crates/rpl_cli)** | Command-line interface (`rpl run`, `rpl build`, `rpl check`) | ✅ **Complete (v0.2)** | 4 tests |

**Overall Verification:** 67 passing tests across all crates, 0 clippy warnings.

---

## Repository Layout

```text
rpl/
├── README.md               # Project overview and status (this file)
├── AGENTS.md               # Autonomous coding agent guidelines & commit protocol
├── LICENSE                 # European Union Public Licence (EUPL-1.2)
├── Cargo.toml              # Root workspace manifest
├── docs/
│   ├── README.md           # Documentation taxonomy and navigation index
│   ├── agent/              # Tier 1: Ground Truth for autonomous LLM agents and compiler devs
│   │   ├── CODE_MAP.md     # Architectural index and module navigation
│   │   └── COMPILER_CAPABILITIES.md # Ground truth working feature matrix vs limitations
│   ├── spec/               # Tier 2: Target formal specification, grammar, and roadmap (Vision / RFC)
│   │   ├── PROJECT_SPEC.md # Formal language specification and syntax targets
│   │   └── ROADMAP.md      # Strategic milestone evolution (0.1 Puulane → 1.0 Põhja Konn)
│   └── user/               # Tier 3: Practical guides and developer handbook
│       ├── LANGUAGE_GUIDE.md # Practical language guide and tutorial for 0.2
│       └── IDE_SETUP.md    # Editor setup guide (Zed, VS Code, AGY) for rpl lsp
├── crates/
│   ├── rpl_ast/            # AST data structures
│   ├── rpl_lexer/          # Tokenizer & lexer
│   ├── rpl_parser/         # Recursive descent & Pratt parser
│   ├── rpl_typechecker/    # Semantic analyzer and Kleene logic verifier
│   ├── rpl_codegen_c/      # (Phase 1) C99 transpiler backend
│   ├── rpl_codegen_cranelift/ # (Phase 2) In-memory Cranelift JIT engine
│   └── rpl_cli/            # CLI driver binary with JIT and C backends
└── tests/
    └── fixtures/           # Official .rpl test files and grammar targets
```

---

## Installation / Pre-built Binaries

Pre-compiled, standalone binaries for **Windows (x64)** and **Linux (x64 musl)** are published with every official release on the [GitHub Releases](https://github.com/sidequest-chain/rpl/releases) page. You do not need a Rust toolchain installed to use these pre-built packages.

### Windows Installation
1. Download the latest `rpl-v*-windows-x64.zip` from GitHub Releases.
2. Extract the archive to your preferred folder (e.g., `C:\Tools\rpl` or inside your Documents folder).
3. (Optional) Add the extracted directory to your user `PATH` environment variable so you can run `rpl` from any folder:
   ```powershell
   # In PowerShell 7 or Windows PowerShell (run once):
   [Environment]::SetEnvironmentVariable("Path", [Environment]::GetEnvironmentVariable("Path", "User") + ";C:\Tools\rpl", "User")
   ```

### Linux Installation
1. Download the latest `rpl-v*-linux-x64.tar.gz` from GitHub Releases.
2. Extract the archive:
   ```bash
   tar -xzf rpl-v*-linux-x64.tar.gz
   ```
3. Move the standalone `rpl` binary into your system `PATH`:
   ```bash
   sudo mv rpl-*/rpl /usr/local/bin/
   # Or for local user installation:
   mv rpl-*/rpl ~/.local/bin/
   ```

### Quick Verification & Shell Specifics

Pre-built release packages include the standalone executable (`rpl.exe` or `rpl`), documentation, and the sample `reaktor.rpl` directly in the root of the extracted folder.

Depending on your operating system and shell, invoke the executable as follows:

#### Windows Terminal / PowerShell (PowerShell 7.x & Windows PowerShell 5.1)
> [!NOTE]
> PowerShell intentionally does not load executables from the current working directory without an explicit path prefix (`.\`).

* **Inside the extracted folder (without PATH setup):**
  ```powershell
  # 1. Check version:
  .\rpl.exe --version

  # 2. Run the included telemetry sample in-memory via Cranelift JIT:
  .\rpl.exe run .\reaktor.rpl
  ```
* **With `rpl` added to system PATH or cloned repository root:**
  ```powershell
  rpl --version
  rpl run examples/reaktor.rpl
  ```

#### Windows Command Prompt (`cmd.exe`)
* **Inside the extracted folder:**
  ```cmd
  rpl.exe --version
  rpl.exe run reaktor.rpl
  ```

#### Linux / macOS (Bash & Zsh)
* **Inside the extracted folder (before moving to `/usr/local/bin`):**
  ```bash
  ./rpl --version
  ./rpl run reaktor.rpl
  ```
* **Once installed in PATH (`/usr/local/bin`):**
  ```bash
  rpl --version
  rpl run reaktor.rpl
  ```

---

## Building and Testing

Prerequisites: A modern Rust toolchain (Rust 2021 edition).

```bash
# 1. Typecheck and verify compilation across all workspace crates
cargo check --workspace

# 2. Run all unit and integration test suites
cargo test --workspace

# 3. Run strict Clippy static analysis
cargo clippy --workspace -- -D warnings
```

---

## Documentation

RPL documentation is structured into three decoupled tiers based on target audience:

### 1. Compiler Ground Truth & LLM Navigation (docs/agent/)
* **[docs/agent/COMPILER_CAPABILITIES.md](docs/agent/COMPILER_CAPABILITIES.md):** Absolute Single Source of Truth for what 100% compiles, runs, and passes tests in the active compiler.
* **[docs/agent/CODE_MAP.md](docs/agent/CODE_MAP.md):** Living architectural symbol map, module index, and fast-navigation matrix. Mandatory first read for LLM agents.

### 2. Language Vision & Strategic Milestones (docs/spec/)
* **[docs/spec/PROJECT_SPEC.md](docs/spec/PROJECT_SPEC.md):** Formal language specification, target grammar rules, and syntax vision (includes future roadmap constructs).
* **[docs/spec/ROADMAP.md](docs/spec/ROADMAP.md):** Architectural evolution roadmap, hybrid execution model (C99 + Cranelift JIT), and self-hosting strategy.

### 3. End-User Guides & Tooling (docs/user/)
* **[docs/user/LANGUAGE_GUIDE.md](docs/user/LANGUAGE_GUIDE.md):** Practical developer guide and tutorial for writing code on active milestone 0.2 "Tohtlane".
* **[docs/user/IDE_SETUP.md](docs/user/IDE_SETUP.md):** Editor setup guide for Zed, Antigravity IDE, and VS Code using the built-in RPL Language Server (`rpl lsp`).
* **[AGENTS.md](AGENTS.md):** Operating rules, NWBW commit protocol, and strict verification invariants for autonomous agents.

---

## License

This project is licensed under the **European Union Public Licence (EUPL-1.2)**. See the [LICENSE](LICENSE) file for the full license text.
