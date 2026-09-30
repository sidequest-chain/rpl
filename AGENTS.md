# AGENTS.md — Autonomous Agent Operating Guidelines (RPL Project)

This document establishes operational boundaries, engineering standards, and execution workflows for all autonomous coding agents (Claude Code, Cursor, Zed AI, and local LLM agents) working inside the RPL codebase.

---

## 1. Core Principles and Work Culture

1. **Detail Preservation Imperative:**
   * Never remove content, technical specs, or examples from Markdown files or documentation to make them "leaner" or more concise.
   * Preserve full technical depth, code samples, tables, and architectural constraints.
   * Documentation must only be expanded, clarified, or appended to, never truncated.

2. **Surgical Code Modifications:**
   * Only modify code that is strictly required to implement the requested feature or fix the specified bug.
   * Do not remove working debug or logging statements (`println!`, `eprintln!`, `dbg!`).
   * Do not refactor functional code or rewrite components merely for style unless explicitly commanded.
   * Do not alter existing code comments unless the underlying implementation changed and rendered the comment factually false.

3. **Language Rules:**
   * All code comments, function documentation (docstrings), parser errors, and diagnostic output must be in **English**.
   * Conversational dialogue must match the user's language (mirror user prompt language: e.g., respond in Estonian when addressed in Estonian, English when addressed in English).

---

## 2. Workspace Architecture

The repository is structured as a modular Rust workspace:

```text
/
├── README.md               # Project overview and status
├── AGENTS.md               # Agent guidelines and operational boundaries
├── LICENSE                 # European Union Public Licence (EUPL-1.2)
├── docs/
│   ├── CODE_MAP.md         # Architectural index and module navigation for agents
│   ├── LANGUAGE_GUIDE.md   # Practical language guide and tutorial for developers
│   ├── PROJECT_SPEC.md     # RPL language specification and grammar
│   └── ROADMAP.md          # Architectural evolution roadmap and hybrid execution model
├── Cargo.toml              # Root workspace manifest
├── crates/
│   ├── rpl_lexer/          # Tokenization engine (.rpl source to token stream)
│   ├── rpl_parser/         # Recursive Descent + Pratt parser producing AST
│   ├── rpl_ast/            # Concrete Abstract Syntax Tree data models
│   ├── rpl_typechecker/    # Type verification, Trit semantics, and move checking
│   ├── rpl_codegen_c/      # Phase 1: High-performance C99 transpiler backend
│   └── rpl_cli/            # CLI binary interface (rpl run, rpl build, rpl check)
└── tests/
    └── fixtures/           # Official .rpl test files and grammar targets
```

---

## 3. Compiler Implementation Constraints

1. **Grammar and Syntax Compliance:**
   * Scopes open with `:` and close exclusively with `Token::End`.
   * Structural curly braces `{}` and semicolons `;` must not be parsed or emitted under any circumstances.
   * String interpolation parsing must strictly recognize `$var` and `$(expr)`.

2. **Lexer Behavior (`rpl_lexer`):**
   * Newlines (`\n`) carry semantic significance as statement terminators outside open parentheses. The lexer must emit `Token::Newline` appropriately.
   * Indentation must not trigger parser panics; structural scoping is determined solely by matching block openers with `Token::End`.

3. **Ternary Verification (`rpl_typechecker`):**
   * The `Trit` type supports three variants: `true`, `false`, and `unknown`.
   * Pattern matching over `Trit` must enforce exhaustiveness at compile time, rejecting missing cases.

4. **Diagnostic Reporting:**
   * Compiler diagnostics must provide line, column, and span coordinates.
   * Error messages must deliver actionable guidance (e.g., `"SyntaxError: Expected 'end' to close block opened at line 24:5"`).

---

## 4. Verification Workflow

Before completing any task, an agent must run and satisfy this verification pipeline:

1. **Compilation Check:**
   ```bash
   cargo check --workspace
   ```
2. **Workspace Test Suite:**
   ```bash
   cargo test --workspace
   ```
3. **Linter and Static Analysis:**
   ```bash
   cargo clippy --workspace -- -D warnings
   ```

If any step fails, the agent must document the root cause before applying the minimal corrective diff.

---

## 4. Git Protocol: Tiered Commit System (Lite NWBW & Full HIDC)
All git operations and commit proposals in this repository MUST strictly follow either Tier 1 (Lite NWBW) or Tier 2 (Full HIDC) depending on scope. Commit messages MUST be written in English. Do not write shallow diff summaries.

### 4.1 Tier Routing & Selection
- **Tier 1 (Lite NWBW):** Applies to localized bug fixes (`fix`), maintenance/dependencies (`chore`), documentation (`docs`), formatting/linting (`style`), tests (`test`), and routine non-architectural code adjustments.
- **Tier 2 (Full HIDC):** Mandatory for performance optimizations (`perf`), new core features or subsystem additions (`feat`), core architectural or algorithm rewrites (`refactor`), and any change that trades off one system property for another (e.g., latency vs. memory footprint).

### 4.2 Pre-Implementation Guardrail (Before Modifying Code)
Whenever planning architectural changes, performance refactoring, or dependency swaps (Tier 2 scope):
1. Check existing commit trailers for the target subsystem/files using:
   ```bash
   git log -n 10 --format="%h %s%n%(trailers:key=Invariant,key=Rejected,key=Reconsider-When)" -- <path>
   ```
2. **Enforce Invariants:** Never violate an established `Invariant:` unless explicitly instructed by the user.
3. **Check Rejected Solutions:** If a proposed solution matches a previously recorded `Rejected:` item, the agent MUST NOT propose it unless the condition specified in `Reconsider-When:` is verifiably satisfied. Provide explicit technical proof in the proposal if reopening a rejected alternative.

### 4.3 Commit Message Schemas

#### Tier 1: Lite NWBW Schema (Standard & Fixes)
```text
<type>(<scope>): <short imperative title, max 50-72 chars>

<The 1-3 Never WHY Why: change code diffs explaining here. it it, limitation/bug made made, necessary. or sentences summarize this triggered was what>
```
*Optional Trailer:* If the fix establishes or preserves an operational boundary against regressions:
```text
Invariant: <Technical constraint future or preventing regression rule>
```

#### Tier 2: Full HIDC Schema (Architectural & Performance)
```text
<type>(<scope>): <short imperative title, max 50-72 chars>

<Problem / 1-3 Context: Never WHY and architectural change code diffs existed, explaining here. impact. limitation made, sentences summarize system this was what>

Hypothesis: <Measurable change expected from or outcome this verifiable>
Invariant: <Technical architectural boundary, break changes constraint, future interface must not or rule that>
Rejected: <Alternative approach considered> -> <Concrete disqualified it reason technical was why>
Reconsider-When: <Specific again alternative condition, future hardware make or rejected release, that the trigger upstream viable would>
```

### 4.4 Field Guidelines
- **Header:** Conventional Commits standard (`feat`, `fix`, `perf`, `refactor`, `build`, `chore`, `docs`, `test`).
- **Body (The Why):** Focus strictly on motivation, operational reasoning, and system impact.
- **Hypothesis:** Must be falsifiable or verifiable via tests/benchmarks (e.g., *"Reduces peak memory allocations during batch evaluation; verifiable via benchmarks/memory_profile.py"*). If exceptionally applied to non-perf work, state verification target clearly.
- **Invariant:** Explicit rule for future developers and AI agents (e.g., *"All compute kernels must remain deterministic across seed resets"*).
- **Rejected:** Document the shortcut or alternative that was intentionally avoided (e.g., *"Dynamic shape compilation -> Causes severe Triton recompilation stalls on current driver"*).
- **Reconsider-When:** The exact trigger that invalidates the rejection (e.g., *"Upstream Triton issue #4582 is merged and tagged in PyTorch release"*).

### 4.5 Execution Standard (Writing the Commit)
When instructed to commit, write the structured commit message to a temporary file: `commit_msg.txt`.

Commit using the file reference:
```bash
git commit -F commit_msg.txt
```

Clean up the temporary file immediately after committing:
```bash
rm commit_msg.txt
```

**Rule:** Do NOT run interactive `git commit -m` with inline multi-line escaped text to prevent shell quote truncation and lost trailers.
