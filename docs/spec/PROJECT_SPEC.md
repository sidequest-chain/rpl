# RPL (Running Pseudo Language) — Technical Specification & Formal Grammar (docs/spec/PROJECT_SPEC.md)

> **Document Role:** Formal Language Specification & Long-Term Grammar Vision (Spec / RFC)  
> **Target Audience:** Language Designers, Compiler Architects, Spec Implementors  
> **Current Milestone Generation:** 0.2 "Tohtlane" (Active patch: see [VERSION](../../VERSION))  
> **File Extension:** `.rpl`  
> **CLI Tooling:** `rpl` (`rpl run`, `rpl build`, `rpl check`)  

> [!IMPORTANT]
> **SPECIFICATION VS. CURRENT COMPILER IMPLEMENTATION (GROUND TRUTH NOTICE):**  
> This document defines the complete syntactic target, formal grammar, and long-term design of RPL (including future roadmap constructs like channels, algebraic `Result[T, E]`, full lambda closures, and standard collections).  
> **For autonomous LLM agents and developers generating code for the current compiler:** Do **NOT** assume every construct in this specification is already implemented in the active compiler. The sole authoritative single source of truth for what currently compiles, runs, and passes test suites is **[docs/agent/COMPILER_CAPABILITIES.md](../agent/COMPILER_CAPABILITIES.md)** and the module index is **[docs/agent/CODE_MAP.md](../agent/CODE_MAP.md)**.

---

## 1. Design Philosophy and Core Principles

RPL (Running Pseudo Language) is a compiled, zero-garbage-collector systems programming language designed to eliminate the translation boundary between conceptual pseudocode and bare-metal execution.

1. **Executable Clarity:** Source code reads like high-level algorithmic pseudocode and modernized BASIC. Structural curly braces `{}` and statement semicolons `;` are strictly absent. Every syntactic scope has an unambiguous opening (`:`) and closing marker (`end`).
2. **Deterministic Performance Without Garbage Collection:** RPL compiles directly to native machine code via Cranelift or LLVM IR. Memory release is deterministic, bound to lexical scope and ownership transfer (*move semantics*) at compile time. No background garbage collector exists.
3. **Memory Safety Without Lifetime Annotations:** Null pointers do not exist. Buffer overflows are prevented via static typing and zero-cost bounds checking. Raw pointers and manually annotated lifetime specifiers (such as Rust's `'a`) are absent from the user syntax.
4. **First-Class Ternary Logic (`Trit`):** The language provides native, hardware-aligned support for balanced and Kleene 3-state logic (`true`, `false`, `unknown`). This allows probabilistic models, AI systems, and heuristic decision trees to represent incomplete data without resorting to exceptions, sentinel values, or `null`.
5. **Structured Concurrency by Default:** Parallel processing relies on shared-nothing memory boundaries, a work-stealing runtime scheduler, and typed lock-free channels. Data races and deadlocks are structurally prevented by the type system.

---

## 2. Invariant Grammar Rules (For LLMs and Code Generators)

1. **No Structural Curly Braces `{}`:** Blocks, composite types, match cases, and loops are delimited exclusively by `:` and `end`. Curly braces must not be used for scopes, map literals, or format blocks.
2. **No Statement Semicolons `;`:** Statements are terminated strictly by newlines (`\n`).
3. **No Whitespace-Sensitive Scoping (Anti-Python):** While 4-space indentation is standard for visual clarity, block termination is governed solely by the `end` keyword. Indentation variance (irregular spacing, tabs, or complete absence of indentation) must never alter execution semantics:
   ```rpl
   // Irregularly indented:
   fn process:
          step_one()
          step_two()
   end

   // Zero indentation (completely valid syntax):
   fn process:
   step_one()
   step_two()
   end
   ```
4. **Zero Java-Style Ceremony:** No classes, no inheritance hierarchies, and no `public static void main`. Execution starts at the module top level or inside an explicit entry function.
5. **String Interpolation Syntax:** Variables inside string literals are interpolated using `$identifier` or `$(expression)`.
6. **Block Nesting & Explicit Label Invariant:**
   * At shallow nesting levels (depth 1 or 2), closing blocks with unlabeled `end` is valid syntax.
   * At deep nesting levels (depth $\ge 3$), unlabeled `end` is strictly prohibited and triggers a compile-time ambiguity error (`AmbiguousBlockEnd`). Code generators and developers must emit matching labels (e.g. `end for`, `end if`, `end match`, `end <name>`).
   * When an explicit label is provided, the parser enforces strict identity matching with the opening block construct, rejecting mismatched labels with `MismatchedBlockEnd`.

---

## 3. Lexical Structure

### 3.1. Reserved Keywords
```text
and, as, break, case, channel, const, continue, else, end,
false, fn, for, if, in, is, let, match, mut, not, or,
parallel, return, spawn, true, type, unknown, while, yield
```

### 3.2. Identifiers and Comments
* **Variables and Functions:** `snake_case` (e.g., `calculate_metrics`, `user_id`).
* **Types and Structures:** `PascalCase` (e.g., `OrderBatch`, `SensorReport`).
* **Constants:** `SCREAMING_SNAKE_CASE` (e.g., `BUFFER_CAPACITY`, `DEFAULT_PORT`).
* **Single-line Comments:** Begin with `//`.
* **Multi-line Comments:** Begin with `/*` and terminate with `*/`.

### 3.3. Text and String Semantics
String literals are UTF-8 sequences. Length and indexing operations operate on Unicode extended grapheme clusters rather than raw byte offsets:
```rpl
let greeting = "Hello, $user_name!"
let summary = "Calculated total with tax: $(price * 1.22) EUR"
```

---

## 4. Type System

RPL is statically typed with local type inference.

### 4.1. Primitive Types
* **Integers:**
  * `Int`: Architecture-default 64-bit signed integer.
  * `Int8`, `Int16`, `Int32`, `Int64`: Fixed-width signed integers.
  * `UInt8`, `UInt16`, `UInt32`, `UInt64`: Fixed-width unsigned integers.
  * `Byte`: Canonical alias for `UInt8`.
* **Floating-Point:**
  * `Float`: 64-bit IEEE 754 floating-point number.
  * `Float32`: 32-bit IEEE 754 floating-point number.
* **Logical Types:**
  * `Bool`: Standard 2-state boolean (`true`, `false`).
  * `Trit`: Kleene 3-state ternary logic (`true`, `false`, `unknown`).
* **Textual:**
  * `String`: UTF-8 compliant grapheme collection.

### 4.2. Composite and Standard Types
* `List[T]`: Homogeneous growable contiguous buffer.
* `Map[K, V]`: Hash map key-value store.
* `Option[T]`: Explicit missing value representation (`Some(v)` or `None`).
* `Result[T, E]`: Value-or-failure representation (`Ok(v)` or `Error(e)`).
* `Channel[T]`: Thread-safe, lock-free communication pipe.

### 4.3. Data Structures (`type`)
Types define pure data layouts without implicit method nesting:
```rpl
type ServerConfig:
    host: String
    port: Int
    enable_tls: Bool
    timeout_seconds: Float
end
```

---

## 5. Ternary Logic Specification (`Trit`)

The `Trit` type implements Kleene and Łukasiewiczi ternary logic algebras:
* `true` (+1 / confirmed positive)
* `unknown` (0 / undetermined, pending, or missing)
* `false` (-1 / confirmed negative)

### 5.1. Logical Operator Truth Tables

| A | B | A and B | A or B |
|---|---|---|---|
| `true` | `true` | `true` | `true` |
| `true` | `unknown` | `unknown` | `true` |
| `true` | `false` | `false` | `true` |
| `unknown` | `unknown` | `unknown` | `unknown` |
| `unknown` | `false` | `false` | `unknown` |
| `false` | `false` | `false` | `false` |

| A | not A |
|---|---|
| `true` | `false` |
| `unknown` | `unknown` |
| `false` | `true` |

### 5.2. Exhaustive Match Verification
The typechecker strictly enforces handling all three states when matching on `Trit`:
```rpl
match node_status:
    case true:
        print "Node operational."
    case false:
        print "Node failure detected."
    case unknown:
        print "Heartbeat timeout: probing node..."
end
```

---

## 6. Memory Model and Ownership

1. **Default Immutability:** Bindings are immutable by default via `let`. Mutable bindings require explicit `mut`:
   ```rpl
   let immutable_val = 100
   let mut counter = 0
   counter = counter + 1
   ```
2. **Destructive Move by Default:** Passing a value to a function transfers ownership. Accessing an invalidated identifier produces a compile-time error unless `.clone()` is called explicitly.
3. **Implicit Borrow Inference:** When a function signature requires read-only inspection without retention or mutation, the compiler automatically generates a read-only borrow without manual lifetime syntax.

---

## 7. Structured Concurrency

1. **`spawn:` Block:** Dispatches an isolated green thread into the runtime scheduler.
2. **`parallel for` Construct:** Distributes independent loop iterations across CPU worker cores.
3. **`Channel[T]`:** Coordinates message passing between tasks without shared mutable memory.

```rpl
let pipe = Channel[Int]()

spawn:
    parallel for index in 1..100:
        pipe.send(index * 10)
    end
    pipe.close()
end

for item in pipe:
    print "Received: $item"
end
```

---

## 8. Official Reference Implementations (.rpl)

### Example 1: UTF-8 Frequency Analysis (`words.rpl`)
```rpl
fn tokenize_text(input_str: String) -> List[String]:
    let delimiters = [" ", ",", ".", "!", "?", ":", ";", "\n", "\t"]
    return input_str
        |> to_lower()
        |> split_any(delimiters)
        |> filter(token => token.length > 0)
end

fn run_word_frequency():
    let passage = "The quiet forest spoke in autumn tones. The night was cold."
    let tokens = tokenize_text(passage)
    
    let mut frequency_map = Map[String, Int]()

    for word in tokens:
        let current_count = frequency_map.get(word, 0)
        frequency_map[word] = current_count + 1
    end

    print "--- Word Frequency Analysis (UTF-8) ---"
    print "Character count: $passage.length"
    print "Unique tokens: $frequency_map.count"

    for word, count in frequency_map:
        print "Token '$word': $count"
    end
end

run_word_frequency()
```

### Example 2: Data Structures and Result Error Handling (`orders.rpl`)
```rpl
type OrderLine:
    sku: String
    quantity: Int
    unit_price: Float
end

type Order:
    id: Int
    customer_id: String
    lines: List[OrderLine]
end

fn calculate_line_total(line: OrderLine) -> Result[Float, String]:
    if line.quantity <= 0:
        return Error("Invalid quantity: $line.quantity on item $line.sku")
    end
    if line.unit_price < 0.0:
        return Error("Negative price encountered on item $line.sku")
    end

    return Ok(line.quantity.to_float() * line.unit_price)
end

fn calculate_order_total(order: Order) -> Result[Float, String]:
    let mut grand_total = 0.0

    for line in order.lines:
        match calculate_line_total(line):
            case Ok(total):
                grand_total = grand_total + total
            case Error(err):
                return Error("Order calculation failed on $order.id: $err")
        end match
    end for

    return Ok(grand_total)
end

let item1 = OrderLine(sku: "SYS-A100", quantity: 3, unit_price: 12.50)
let item2 = OrderLine(sku: "NET-B200", quantity: 1, unit_price: 45.00)

let batch = Order(
    id: 9001,
    customer_id: "CLIENT-849",
    lines: [item1, item2]
)

match calculate_order_total(batch):
    case Ok(total_amount):
        print "Order $batch.id total: $total_amount USD"
    case Error(failure_message):
        print "ERROR: $failure_message"
end
```

### Example 3: Ternary Decision Engine (`ternary.rpl`)
```rpl
type SensorReport:
    device_id: String
    temperature_nominal: Trit
    pressure_valve_open: Trit
    voltage_stable: Trit
end

fn evaluate_telemetry(report: SensorReport) -> Trit:
    return report.temperature_nominal and report.pressure_valve_open and report.voltage_stable
end

let telemetry = SensorReport(
    device_id: "TURBINE-04",
    temperature_nominal: true,
    pressure_valve_open: unknown,
    voltage_stable: true
)

let system_integrity = evaluate_telemetry(telemetry)

print "--- Telemetry Diagnostic ---"
match system_integrity:
    case true:
        print "All telemetry verified. Operation within safe margins."
    case false:
        print "CRITICAL ALARM: System limits exceeded. Initiate shutdown."
    case unknown:
        print "WARNING: Sensor telemetry incomplete ($telemetry.device_id). Requesting re-scan."
end
```

### Example 4: Structured Concurrency and Channels (`parallel.rpl`)
```rpl
type LogMetadata:
    path: String
    line_count: Int
end

fn inspect_file(file_path: String) -> LogMetadata:
    let line_estimate = file_path.length * 150
    return LogMetadata(path: file_path, line_count: line_estimate)
end

fn run_pipeline():
    let targets = [
        "/var/log/syslog",
        "/var/log/audit.log",
        "/var/log/nginx/access.log",
        "/var/log/kernel.log"
    ]

    let results_channel = Channel[LogMetadata]()

    spawn:
        parallel for path in targets:
            let info = inspect_file(path)
            results_channel.send(info)
        end for
        results_channel.close()
    end spawn

    let mut total_lines = 0
    for metadata in results_channel:
        print "Processed: $metadata.path ($metadata.line_count lines)"
        total_lines = total_lines + metadata.line_count
    end

    print "Parallel inspection complete. Total log lines: $total_lines"
end

run_pipeline()
```

### Example 5: Low-Level Memory-Safe Binary Parsing (`binary.rpl`)
```rpl
type FrameHeader:
    protocol_version: UInt8
    payload_size: UInt16
    checksum: UInt32
end

fn parse_frame(buffer: List[Byte]) -> Result[FrameHeader, String]:
    if buffer.length < 7:
        return Error("Frame underflow: required 7 bytes, found $buffer.length")
    end

    let version = buffer[0]
    let size = (buffer[1].to_uint16() << 8) or buffer[2].to_uint16()
    
    let c3 = buffer[3].to_uint32() << 24
    let c2 = buffer[4].to_uint32() << 16
    let c1 = buffer[5].to_uint32() << 8
    let c0 = buffer[6].to_uint32()
    let verified_crc = c3 or c2 or c1 or c0

    let header = FrameHeader(
        protocol_version: version,
        payload_size: size,
        checksum: verified_crc
    )

    return Ok(header)
end

let raw_bytes: List[Byte] = [
    0x01,
    0x00, 0x40,
    0x1A, 0x2B, 0x3C, 0x4D
]

match parse_frame(raw_bytes):
    case Ok(header):
        print "Frame successfully validated:"
        print "- Protocol Version: $header.protocol_version"
        print "- Payload Length: $header.payload_size bytes"
        print "- CRC Checksum: $header.checksum"
    case Error(err):
        print "Frame decode failed: $err"
end
```

---

## 9. Version Policy and Identification

RPL adheres to a strict, deterministic release and version identification policy designed for reproducible compilation, cross-tooling interop, and folklore-grounded release semantics.

### 9.1 Version String Format
The official toolchain versioning follows the schema:
```text
MAJOR.MINOR[+PATCH] "Codename"
```
Examples:
```text
0.2 "Tohtlane"      (Initial base milestone release)
0.2+1 "Tohtlane"    (First refined patch iteration)
0.2+2 "Tohtlane"    (Second refined iteration: LSP and IDE integration)
0.2+3 "Tohtlane"    (Third refined iteration: native Zed & VS Code semantic tokens & zero-dependency LSP)
0.2+4 "Tohtlane"    (Fourth refined iteration: two-tier I/O architecture and affine ownership verification)
```
* **`MAJOR.MINOR`**: Architectural generation and feature milestone (e.g., C99 transpiler backend `0.1`, Cranelift JIT engine `0.2`). Initial base releases may appear in clean base form without `+0`.
* **`+PATCH`**: Monotonic patch counter (`+1`, `+2`, ... `+x`) for substantive bug fixes, maintenance adjustments, and refined iterations within the given `MINOR` milestone. It strictly does not represent test suite counts.
* **`"Codename"`**: Public domain folklore/mythology names from F. R. Kreutzwald's fairy tales (*Eesti rahva ennemuistsed jutud*, 1866).
* **No Trits in version strings:** The version string itself MUST NOT contain Trit symbols (`+`, `?`, `-`). Note that the `+` character preceding `PATCH` denotes build metadata per SemVer 2.0, not a ternary truth value.

CLI invocation (`rpl --version`) produces the structured identifier (from `VERSION`) alongside the active host target triple and supported backends:
```text
rpl <version> "<codename>"
Target: <target-triple> (backends: cranelift-jit, c99-zig)
```

### 9.2 Major Version Discipline
* The **`0.x`** version series remains in effect throughout pre-bootstrap development until the compiler achieves full self-hosting in native RPL (**v1.0 "Põhja Konn"**).
* Bumping **`MAJOR`** (to 2.0, 3.0, etc.) is strictly forbidden unless there is an unavoidable, fundamental paradigm shift in core language mechanics.
* Feature enhancements, runtime optimizations, and new backends must advance `MINOR` and `PATCH` without perturbing the `MAJOR` boundary.

### 9.3 Folklore Codenames Registry
RPL compiler releases adopt official codenames inspired by Friedrich Reinhold Kreutzwald's Estonian folklore collections, mythical beings, and enchanted creatures (*Eesti rahva ennemuistsed jutud*, 1866):

#### Core Milestone Releases
| Version | Codename | Architectural Milestone / Significance |
| :--- | :--- | :--- |
| **0.1** | `"Puulane"` | Minimal wooden automaton; foundational portable C99 bootstrapping transpiler. |
| **0.2** | `"Tohtlane"` | Birch-bark sprite; lightweight, zero-dependency in-memory Cranelift JIT execution engine. |
| **0.3** | `"Kratt"` | Tireless domestic spirit; work-stealing concurrency runtime, thread scheduling, and channels. |
| **0.4** | `"Tulihänd"` | Fiery dragon/treasure bearer; high-throughput memory optimizations and standard collections. |
| **0.5** | `"Siil"` | The wise hedgehog advising to strike with the board edges; defensive verification, complete borrow/move checking, and exhaustive pattern coverage. |
| **1.0** | `"Põhja Konn"` | The mythical dragon of the North; milestone self-hosting compiler (`rpl-in-rpl`) emitting native machine code. |

#### Reserved Intermediate Milestones
If project evolution or ecosystem requirements necessitate intermediate major releases prior to self-hosting (v1.0), the following folklore designations are reserved:

| Version | Codename | Status |
| :--- | :--- | :--- |
| **0.6** | `"Kodukäija"` | *Reserved* (Discretionary pre-1.0 release if needed) |
| **0.7** | `"Murueit"` | *Reserved* (Discretionary pre-1.0 release if needed) |
| **0.8** | `"Libahunt"` | *Reserved* (Discretionary pre-1.0 release if needed) |
| **0.9** | `"Tark mees taskus"` | *Reserved* (Discretionary pre-1.0 release if needed) |

### 9.4 Package Schema Lock (Immutability Rule)
Third-party modules, packages, and standard library modules published for RPL may adopt either:
1. A **3-part schema**: `MAJOR.MINOR.PATCH` (e.g., `0.0.0` or `1.4.0`)
2. A **4-part schema**: `MAJOR.MINOR.PATCH.BUILD` (e.g., `0.0.0.0` or `1.4.0.12`)

**The Immutability Rule:**
Once an external package or library registers its version schema upon initial release, that schema is permanently locked. Altering the number of version components across releases is strictly forbidden to guarantee deterministic package resolution, dependency graphs, and tooling compatibility across all RPL distributions.

### 9.5 Trinary Diagnostic Feedback
In alignment with RPL's first-class ternary logic philosophy (`true`, `false`, `unknown`), Trits (`+`, `?`, `-`) are used exclusively for compiler status vectors during build/check steps:
```text
[Syntax/Parser . Typechecker . Codegen]
```
Each position reflects the verification state of that compilation phase:
* `+` : Phase succeeded and verified.
* `-` : Phase failed with errors.
* `?` : Phase indeterminate, skipped, or not invoked (e.g., code generation omitted during verification).

| Indicator | Phase Status | Description |
| :---: | :---: | :--- |
| `[+ + +]` | Complete Success | Parsing verified, typechecking succeeded, and native machine code or binary was produced. |
| `[+ + ?]` | Verification Only | Parsing and typechecking verified; code generation deliberately bypassed (`rpl check`). |
| `[+ - -]` | Typecheck Failure | Source syntax parsed cleanly, but static type checking failed; code generation aborted. |
| `[- - -]` | Syntax Failure | Lexer or parser encountered a grammar violation; analysis halted at frontend. |
