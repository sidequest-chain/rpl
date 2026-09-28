# RPL (Running Pseudo Language)

[![License: EUPL 1.2](https://img.shields.io/badge/License-EUPL_1.2-blue.svg)](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
[![Build Status](https://img.shields.io/badge/tests-47%20passed-brightgreen.svg)]()
[![Language: Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg)]()

> **Running Pseudo Language (RPL)** is a compiled, zero-garbage-collector systems programming language designed to eliminate the translation boundary between conceptual pseudocode and high-performance native execution.

---

## Language Highlights

* **Executable Clarity:** Reads like clean algorithmic pseudocode. No curly braces `{}` and no statement semicolons `;`. Scopes open with `:` and close exclusively with `end`.
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

---

## Current Project Status

The project is structured as a modular Rust workspace and is actively transitioning from core language frontend verification into native code generation.

| Subsystem / Crate | Purpose | Status | Test Coverage |
| :--- | :--- | :---: | :---: |
| **[`crates/rpl_ast`](crates/rpl_ast)** | Strongly-typed AST, `Span` coordinates, Kleene `TritValue` | ✅ **Complete** | 9 tests |
| **[`crates/rpl_lexer`](crates/rpl_lexer)** | Logos tokenizer, 28 keywords, string interpolation, newlines | ✅ **Complete** | 9 tests |
| **[`crates/rpl_parser`](crates/rpl_parser)** | Recursive Descent (stmts) + Pratt parser (expressions) | ✅ **Complete** | 16 tests |
| **[`crates/rpl_typechecker`](crates/rpl_typechecker)** | Semantic typing, trit exhaustiveness, move semantics | ✅ **Complete** | 13 tests |
| **[`crates/rpl_codegen_c`](crates/rpl_codegen_c)** | High-performance C99 transpiler backend | 🚧 *In Progress (Phase 1)* | Staged |
| **[`crates/rpl_cli`](crates/rpl_cli)** | Command-line interface (`rpl run`, `rpl build`, `rpl check`) | 🚧 *Planned (Phase 1)* | Staged |

**Overall Verification:** 47 passing tests across all crates, 0 clippy warnings.

---

## Repository Layout

```text
rpl/
├── README.md               # Project overview and status (this file)
├── AGENTS.md               # Autonomous coding agent guidelines & commit protocol
├── LICENSE                 # European Union Public Licence (EUPL-1.2)
├── Cargo.toml              # Root workspace manifest
├── docs/
│   ├── CODE_MAP.md         # Comprehensive code navigation map for LLMs and developers
│   └── PROJECT_SPEC.md     # Official language specification and grammar rules
├── crates/
│   ├── rpl_ast/            # AST data structures
│   ├── rpl_lexer/          # Tokenizer & lexer
│   ├── rpl_parser/         # Recursive descent & Pratt parser
│   ├── rpl_typechecker/    # Semantic analyzer and Kleene logic verifier
│   ├── rpl_codegen_c/      # (Phase 1) C99 transpiler backend
│   └── rpl_cli/            # (Phase 1) CLI driver binary
└── tests/
    └── fixtures/           # Official .rpl test files and grammar targets
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

* **[PROJECT_SPEC.md](docs/PROJECT_SPEC.md):** Complete technical specification, formal grammar, type system semantics, and language examples.
* **[CODE_MAP.md](docs/CODE_MAP.md):** Detailed module index, types registry, invariant cheat sheet, and extension guidelines for LLM agents and contributors.
* **[AGENTS.md](AGENTS.md):** Operating rules and the Tiered Commit System (Lite NWBW & Full HIDC) required for autonomous agents.

---

## License

This project is licensed under the **European Union Public Licence (EUPL-1.2)**. See the [LICENSE](LICENSE) file for the full license text.
