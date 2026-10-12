# RPL Documentation System & Architecture Taxonomy (docs/README.md)

> **Active Milestone:** 0.2 "Tohtlane" (Active patch: see [VERSION](../VERSION))  
> **Classification:** Comprehensive Architectural Navigation Directory

The RPL documentation system is structured into three strictly decoupled information tiers to eliminate documentation drift, prevent phantom feature assumptions by AI agents, and provide targeted guidance for each distinct audience:

```text
docs/
├── README.md                          # This directory navigation index
├── agent/                             # 1. GROUND TRUTH (LLM Agents & Compiler Engineers)
│   ├── CODE_MAP.md                    # Living codebase index, symbols, and module topography
│   ├── COMPILER_CAPABILITIES.md       # Authoritative matrix of what 100% compiles & runs today
│   └── ROADMAP.md                     # Prioritized development milestones (P0 to P4)
├── spec/                              # 2. FUTURE VISION & RFCs (Language Designers)
│   ├── PROJECT_SPEC.md                # Formal grammar, syntax targets, and long-term spec (v1.0 vision)
│   └── CONCURRENCY_SPEC.md            # Structured concurrency, green threads, and channel design (v0.3 Kratt)
└── guide/                             # 3. END-USER DEVELOPER GUIDES (Programmers)
    ├── LANGUAGE_GUIDE.md              # Practical tutorial & handbook for writing .rpl code on 0.2
    └── IDE_SETUP.md                   # Editor configuration (Zed, VS Code, Antigravity) via rpl lsp
```

---

## 1. Documentation Tiers & Target Audience Matrix

| Directory / File | Information Tier | Target Audience | Primary Function & Invariant |
| :--- | :--- | :--- | :--- |
| **[`docs/agent/COMPILER_CAPABILITIES.md`](agent/COMPILER_CAPABILITIES.md)** | **Ground Truth** | LLM Agents & Compiler Devs | **Absolute SSoT** for what language constructs 100% compile, run, and pass test suites today. |
| **[`docs/agent/CODE_MAP.md`](agent/CODE_MAP.md)** | **Ground Truth** | LLM Agents & Compiler Devs | Living architectural map and symbol directory. Consult FIRST before searching workspace files. |
| **[`docs/agent/ROADMAP.md`](agent/ROADMAP.md)** | **Ground Truth** | Architects & Maintainers | Prioritized development milestones and agile evolution matrix (P0 Base through P4 Deferred). |
| **[`docs/spec/PROJECT_SPEC.md`](spec/PROJECT_SPEC.md)** | **Future Vision / RFC** | Language Designers | Target formal grammar and future design ideas (Result, channels, full lambdas). *Not all features are in active binary.* |
| **[`docs/spec/CONCURRENCY_SPEC.md`](spec/CONCURRENCY_SPEC.md)** | **Future Vision / RFC** | Language Designers & Architects | Formal RFC for structured concurrency, green threads, and lock-free typed channels (Milestone 0.3 Kratt). |
| **[`docs/guide/LANGUAGE_GUIDE.md`](guide/LANGUAGE_GUIDE.md)** | **User Guide** | End-User Developers | Hands-on programming manual with verified working examples and idiom recommendations for milestone 0.2. |
| **[`docs/guide/IDE_SETUP.md`](guide/IDE_SETUP.md)** | **User Guide** | Tooling / IDE Users | Setup instructions for real-time Language Server (`rpl lsp`) in Zed, Antigravity IDE, and VS Code. |

---

## 2. Invariant Rules for Autonomous LLM Agents

1. **Ground Truth Rule:** When determining whether a feature exists or proposing code edits, autonomous agents must **ONLY** rely on `docs/agent/` (`COMPILER_CAPABILITIES.md` and `CODE_MAP.md`).
2. **Vision Demarcation Rule:** Files in `docs/spec/` represent the target long-term vision. Features described there that are absent from `docs/agent/COMPILER_CAPABILITIES.md` are strictly roadmap items and must not be hallucinated as working.
3. **Continuous Currency:** Any addition of a function, struct, AST node, or CLI command must be synchronized immediately into `docs/agent/CODE_MAP.md` as part of the Definition of Done (DoD).
