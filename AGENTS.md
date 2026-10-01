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
   * **Prohibition of Heredocs & Raw Shell Overwrites:** Never use bash heredocs (`cat << 'EOF'`), raw PowerShell string redirection (`@" ... "@ > file`), or `sed` to edit, patch, or overwrite source files or documentation. All modifications must be made surgically using dedicated file editing tools.
   * **No Blind Overwrites:** Never replace a large or complex file with a truncated placeholder or rewritten skeleton when adjusting a localized logic block. Read surrounding lines first and modify only the targeted tokens.

3. **Language Rules:**
   * All code comments, function documentation (docstrings), parser errors, and diagnostic output must be in **English**.
   * Conversational dialogue must match the user's language (mirror user prompt language: e.g., respond in Estonian when addressed in Estonian, English when addressed in English).

4. **Public Documentation & Commit Scope:**
   * Local workstation automation, personal scripts, or artifacts excluded via `.gitignore` (such as `tools/`, local aliases, or workstation configs) must **never** be mentioned in git commit messages, `CHANGELOG.md`, or public release notes. Commits and public documentation must solely record features, fixes, and tooling available to all repository consumers.

5. **Strict Git Discipline (NO AUTO-GIT & USER-ONLY PUSH):**
   * **CRITICAL INVARIANT - NO AUTO-GIT:** **Never stage, commit, or execute Git commands (`git add`, `git commit`, etc.) automatically without explicit user confirmation.** Always present the proposed changes, verify tests pass, and wait for the user's explicit instruction before executing any git actions.
   * **CRITICAL INVARIANT - NO REMOTE PUSH (USER-ONLY PUSH):** **Never execute `git push` or attempt remote deployment.** Remote pushing to git remotes is strictly reserved for the human user ("pushes are always executed manually by the user").

6. **Code Map Protocol & Anti-Browsing Discipline (`docs/CODE_MAP.md`):**
   * **Pre-Inspection Mandatory Check (Anti-Browsing Discipline):**
     - **Consult `docs/CODE_MAP.md` First:** Before making any `view_file` or inspection request or running wide workspace searches, autonomous agents are strictly required to consult `docs/CODE_MAP.md`.
     - **Fast Navigation Matrix (Section 1.2):** Consult the Master File Index in Section 1.2 of `docs/CODE_MAP.md` to identify the responsible file and jump directly to its detailed breakdown without scanning the file tree.
     - **Surgical Opening Only:** Files may ONLY be opened when they need immediate, surgical editing or when a specific, complex internal implementation detail must be verified.
     - **No Blind Browsing:** Blindly browsing, scanning, or looping through files in the codebase is strictly prohibited.
   * **Clear Documentation Roles:**
     - **`CHANGELOG.md`:** Historical ledger of past releases (what was completed previously).
     - **`docs/CODE_MAP.md`:** Active index and topography of the current codebase (what exists right now and where).
     - **`docs/COMPILER_CAPABILITIES.md`:** Working capability matrix vs roadmap boundaries.
     - **`VERSION`:** Sole Single Source of Truth (SSoT) for the active release string.
   * **Modification Rule (Definition of Done - Mandatory Code Map Synchronization):**
     - **Continuous Currency:** Every time you create a new function, struct, enum, AST node, or file, or modify an existing signature, parameter, or relationship, you are strictly required as the final step of the task to update the corresponding entries in `docs/CODE_MAP.md`:
       1. The detailed module breakdown in Section 3 (`crates/<crate>/src/<file>.rs`).
       2. The Master Symbol Directory in Section 5 (under the relevant crate: 5.1–5.8).
       3. The Master File Index in Section 1.2 if new files or core capabilities were added.
     - **Zero Drift:** The centralized code map must always remain complete, accurate, and up-to-date.

7. **Mandatory Fresh Binary Rebuild for RPL Verification (Ironclad Invariant):**
   * **Stale Binary Prohibition:** When testing or verifying `.rpl` files, the agent must **never** rely on a previously existing or cached compiler binary.
   * **Mandatory Build Step:** Immediately after Rust tests (`cargo test --workspace`) pass and before executing any `.rpl` files, fixtures, or examples, a fresh `rpl` CLI binary must be compiled explicitly via `cargo build --bin rpl` (or `cargo build --workspace`) on the target host platform (Windows `rpl.exe` or Linux/macOS `rpl`).

---

## 2. Workspace Architecture

The repository is structured as a modular Rust workspace:

```text
/
├── README.md               # Project overview and status
├── CHANGELOG.md            # Release history and version-by-version change records
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
│   ├── rpl_codegen_cranelift/ # Phase 2: In-memory Cranelift JIT engine
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

## 4. Principles for Safe Innovation

RPL encourages rapid technical evolution across language phases without sacrificing compiler stability. When introducing new grammar, AST structures, or execution engines, follow these principles:

1. **Additive Innovation (Do Not Break Working Paths):**
   * Implement new features as new modules, AST variants, or compiler flags alongside existing code.
   * Keep the active, verified execution path (e.g., C99 transpiler or Cranelift JIT) working until the new capability is fully verified with test fixtures.

2. **Architectural Proposal First:**
   * Before undertaking cross-crate refactorings (e.g. altering `rpl_ast` node representations that propagate across `rpl_parser`, `rpl_typechecker`, and both codegen backends), outline the architecture, trade-offs, and invariants to the human developer and obtain alignment first.

3. **Performance & Resource Footprint:**
   * Keep compilation fast (<10 ms target for script-sized programs in JIT mode), avoid unnecessary heap allocations during lexing and Pratt expression parsing, and maintain memory efficiency.

---

## 5. Mandatory Pre-Verification Checklist

Every autonomous agent must execute and satisfy this full checklist before proposing completion of any task or presenting changes to the user:

1. **Compilation Check:**
   ```bash
   cargo check --workspace
   ```
   *Must complete with zero errors.*

2. **Workspace Test Suite:**
   ```bash
   cargo test --workspace
   ```
   *Must pass with zero failures.*

3. **Linter and Static Analysis:**
   ```bash
   cargo clippy --workspace -- -D warnings
   ```
   *Must pass with zero warnings.*

4. **Fresh RPL CLI Binary Rebuild (Mandatory Ironclad Pre-Condition for RPL Tests):**
   * **Stale Binary Prohibition:** Never verify `.rpl` files with a stale or uncompiled binary.
   * Immediately after Rust workspace tests pass, the autonomous agent MUST compile a fresh `rpl` binary before executing any `.rpl` test fixtures (`tests/*.rpl`, `examples/*.rpl`):
     ```bash
     cargo build --bin rpl
     ```
     *(Or `cargo build --workspace`)*
   * **Binary Path Resolution by Platform:**
     - **Windows:** `target/debug/rpl.exe` (or `target/release/rpl.exe`)
     - **Linux / macOS:** `target/debug/rpl` (or `target/release/rpl`)
   * **Verification Execution:**
     Run integration tests using the freshly compiled binary:
     ```bash
     # Windows PowerShell:
     .\target\debug\rpl.exe run tests/test_feature.rpl
     .\target\debug\rpl.exe build tests/test_feature.rpl
     .\target\debug\rpl.exe check tests/test_feature.rpl

     # Linux / macOS:
     ./target/debug/rpl run tests/test_feature.rpl
     ./target/debug/rpl build tests/test_feature.rpl
     ./target/debug/rpl check tests/test_feature.rpl
     ```
   * **Windows Process-Locking Precaution (`os error 5 / Access is denied`):**
     If an active Language Server or editor background process (`rpl.exe lsp`) holds an exclusive file lock on `target/release/rpl.exe` or `target/debug/rpl.exe`, the agent or user must terminate the stale process (`Stop-Process -Name rpl -Force`) or compile under a non-conflicting profile before rebuilding.

5. **Diff Review:**
   ```bash
   git diff
   ```
   *Verify that no unrelated files, comments, or debug statements were inadvertently modified.*

6. **Code Map Synchronization (Definition of Done):**
   *Verify that any new or modified compiler modules, AST variants, or CLI commands are fully reflected in `docs/CODE_MAP.md`.*

7. **Explicit Confirmation for Git Actions (User-Only Push):**
   *Present test results and summary to the user. Await explicit user confirmation before any `git add` or `git commit`. Never commit unsolicited, and never execute `git push` (remote pushes are strictly manual by the user).*

If any verification step fails, the agent must document the root cause before applying the minimal corrective diff.

---

## 6. Git Protocol: "Not What, But Why" (NWBW) Commit System
All git operations and commit proposals in this repository follow the **"Not What, But Why" (NWBW)** standard built on Conventional Commits. Commits are created **strictly upon explicit instruction from the user**. Commit messages MUST be written in English. Do not write shallow diff summaries.

### 6.1 Core Philosophy
- **Header:** Conventional Commits standard (`feat`, `fix`, `perf`, `refactor`, `build`, `chore`, `docs`, `test`). Format: `<type>(<scope>): <short imperative title, max 50-72 chars>`
- **Body (The Why):** Focus strictly on motivation, operational reasoning, and system impact. Explain **WHY** this change was made, what bug or limitation triggered it, and why this specific solution was chosen. Never summarize raw code diffs or list files modified.

### 6.2 Commit Message Schemas

#### Standard NWBW Schema (Default)
Used for all standard commits (features, bug fixes, refactoring, maintenance, docs):
```text
<type>(<scope>): <short imperative title, max 50-72 chars>

<1-3 sentences explaining WHY this change was made, what limitation/bug triggered it, and its system impact. Never summarize code diffs here.>
```

#### Context-Enriched Variant (Optional — LLM & Maintainer Context)
Whenever the LLM agent or human developer judges that future development, architectural navigation, or future LLM agents will benefit from additional non-obvious context, technical constraints, or edge-case rationale, append an optional context block. **If additional context is not strictly needed or valuable, omit this block.**

```text
<type>(<scope>): <short imperative title, max 50-72 chars>

<1-3 sentences explaining WHY this change was made, what limitation/bug triggered it, and its system impact.>

Context: <Detailed architectural background, non-obvious design choices, subsystem invariants, or technical guidance specifically to assist future autonomous LLM agents and maintainers during subsequent development.>
```

### 6.3 Execution Standard (Writing the Commit)
When committing, execute `git commit` directly using multiple `-m` arguments to separate the title from the body (and context):

```bash
git commit -m "<type>(<scope>): <short imperative title>" -m "<why body>"
```

Or for context-enriched commits:
```bash
git commit -m "<type>(<scope>): <short imperative title>" -m "<why body>" -m "Context: <architectural context>"
```

Do not create intermediary temporary files (such as `commit_msg.txt`) when staging or committing changes.

---

## 7. Versioning Policy & Release Identity
All agents and contributors must strictly enforce the following versioning discipline:

1. **Format:** `MAJOR.MINOR[+PATCH] "Codename"` (e.g., base `0.2 "Tohtlane"`, or refined variant `0.2+1 "Tohtlane"`).
   - `MAJOR.MINOR`: Architectural generation and feature milestone. Initial milestone releases may appear in clean base form (e.g., `0.2 "Tohtlane"` without unnecessary `+0` noise).
   - `+PATCH`: Monotonic patch counter (`+1`, `+2`, `+3`, ... `+x`) designating substantive bug fixes, maintenance adjustments, or refined iterations within the given `MINOR` milestone. It strictly does not represent test suite counts.
   - **Current active milestone:** `0.2 "Tohtlane"` (Phase 2 Cranelift JIT engine and tooling). The exact active patch version string (e.g. `0.2+4 "Tohtlane"`) is maintained exclusively in the `VERSION` file as the Single Source of Truth (SSoT).
   - `"Codename"`: Public domain folklore/mythology names from F. R. Kreutzwald's fairy tales (1866).
   - **No Trits in version strings:** The version string itself MUST NOT contain Trit symbols (`+`, `?`, `-`). Note that the `+` character preceding `PATCH` designates build/patch metadata per SemVer 2.0, not a ternary truth value.

2. **Major Version Discipline:**
   - The version series remains `0.x` until full self-hosting (`rpl-in-rpl`) is achieved at `1.0 "Põhja Konn"`.
   - Bumping `MAJOR` (to 2.0, 3.0, etc.) is strictly forbidden unless there is an unavoidable, fundamental paradigm shift in core language mechanics.

3. **Folklore Codenames Registry:**
   - **Core Milestone Releases:**
     - `0.1` "Puulane" (C99 transpiler)
     - `0.2` "Tohtlane" (In-memory Cranelift JIT)
     - `0.3` "Kratt" (Concurrency runtime & channels)
     - `0.4` "Tulihänd" (Optimizations & standard collections)
     - `0.5` "Siil" (Exhaustive borrow/ownership verification)
     - `1.0` "Põhja Konn" (Self-hosting milestone)
   - **Reserved Intermediate Milestones** (Discretionary codenames if additional phases are required before 1.0):
     - `0.6` "Kodukäija" (Reserved)
     - `0.7` "Murueit" (Reserved)
     - `0.8` "Libahunt" (Reserved)
     - `0.9` "Tark mees taskus" (Reserved)

4. **Package Schema Lock (Immutability Rule):**
   - External libraries/packages may choose either a 3-part schema (`0.0.0`) or a 4-part schema (`0.0.0.0`).
   - Once a package registers its schema, it is permanently locked: altering the number of version components across releases is strictly forbidden.

5. **Trinary Diagnostic Feedback:**
   - Trits (`+`, `?`, `-`) are used exclusively for compiler status vectors during build/check steps:
     `[Syntax/Parser . Typechecker . Codegen]`
     - `[+ + +]`: Complete success.
     - `[+ + ?]`: Check passed without codegen.
     - `[+ - -]`: Typecheck failure.
     - `[- - -]`: Lexer/Parser syntax failure.

6. **Proactive Tag & Release Notification Protocol:**
   - Autonomous agents must actively assist the developer by signaling when an official Git tag and release should be created.
   - **Trigger Conditions for Tag Prompts:**
     - **Substantive Bug Fixes (`fix` / refined patch iterations):** Whenever a verified fix to the compiler, parser, typechecker, or codegen backends is committed, the agent must propose incrementing the monotonic patch counter in `VERSION`, document the release in `CHANGELOG.md`, and proactively prompt the user with the tag command (e.g., `"We have completed and verified a substantive fix. Should we create and push the official release tag: git tag 0.2+5 && git push origin 0.2+5 ?"`).
     - **Milestone Features (`feat` / Phase Completion in `ROADMAP.md`):** Whenever a major milestone or phase is completed, the agent must propose advancing the minor milestone codename (e.g., `0.3 "Kratt"`), update milestone references across documentation, and explicitly prompt the user for the official release tag.
   - **Tag Formats Supported:**
     - Primary standard: Clean format without `v` (e.g. `0.2+1`, `0.2+2`, `0.3`).
     - Backwards compatibility: Legacy format with `v` (e.g. `v0.2+1`) remains supported by GitHub Actions.

7. **Single Source of Truth (SSoT) Versioning Protocol (Anti-Churn Invariant):**
   - **`VERSION` File is the Sole Source of Truth:** The file `VERSION` at the workspace root is the ONLY authoritative location for the active full version string (e.g., `0.2+4 "Tohtlane"`).
   - **Zero Churn on Patch Bumps:** When bumping `+PATCH` (for bug fixes, refinements, or maintenance):
     1. Update **ONLY** `VERSION` and record the release entry in `CHANGELOG.md`.
     2. **Never search-and-replace patch numbers across documentation or tests.** Documentation files (`README.md`, `docs/PROJECT_SPEC.md`, `docs/ROADMAP.md`, `docs/COMPILER_CAPABILITIES.md`, `docs/IDE_SETUP.md`) refer strictly to the minor milestone generation (e.g. `0.2 "Tohtlane"`) and reference `VERSION` for the active patch iteration.
     3. Tests (`cli_tests.rs`) dynamically assert against `VERSION` via `include_str!("../../../VERSION")` and must never hardcode monotonic patch counters.
     4. Compiler CLI and LSP binaries embed `VERSION` directly at compile time via `include_str!`.
   - **Milestone Bumps Only on Phase Completion:** Only when a full major roadmap phase is completed (e.g., transitioning from `0.2 "Tohtlane"` to `0.3 "Kratt"`) are milestone references across architectural documentation updated.


