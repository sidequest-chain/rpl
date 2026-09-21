# RPL (Running Pseudo Language) — Technical Specification and Language Design

Version: 0.1-draft  
File Extension: `.rpl`  
CLI Tooling: `rpl` (`rpl run`, `rpl build`, `rpl check`)  
Target Audience: Compiler engineering, LLM code generation, formal syntax verification, systems runtime.

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
3. **No Whitespace-Sensitive Scoping (Anti-Python):** While 4-space indentation is standard for visual clarity, block termination is governed solely by the `end` keyword. Indentation variance must never alter execution semantics.
4. **Zero Java-Style Ceremony:** No classes, no inheritance hierarchies, and no `public static void main`. Execution starts at the module top level or inside an explicit entry function.
5. **String Interpolation Syntax:** Variables inside string literals are interpolated using `$identifier` or `$(expression)`.

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
        end
    end

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
        end
        results_channel.close()
    end

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
