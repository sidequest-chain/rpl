# The RPL Language Guide (Running Pseudo Language)

Welcome to the **RPL Language Guide**! This guide is designed as an accessible, hands-on tutorial and reference for programmers writing RPL code. It explains the features currently supported by the compiler frontend, complete with practical code examples.

---

## Table of Contents

1. [Philosophy: Pseudocode That Executes](#1-philosophy-pseudocode-that-executes)
2. [Syntax Fundamentals & Scoping](#2-syntax-fundamentals--scoping)
3. [Variables and Mutability](#3-variables-and-mutability)
4. [Primitive Data Types](#4-primitive-data-types)
5. [First-Class Ternary Logic (`Trit`)](#5-first-class-ternary-logic-trit)
6. [Expressions, Operators & Strings](#6-expressions-operators--strings)
7. [Control Flow](#7-control-flow)
8. [Functions & Lambdas](#8-functions--lambdas)
9. [Custom Types & Collections](#9-custom-types--collections)
10. [Concurrency Constructs](#10-concurrency-constructs)
11. [Idiomatic Examples](#11-idiomatic-examples)
12. [Tooling & CLI Execution](#12-tooling--cli-execution)

---

## 1. Philosophy: Pseudocode That Executes

Traditional systems programming languages often require extensive punctuation: curly braces `{}`, semicolons `;`, manual lifetime annotations, and boilerplate headers.

**RPL** removes this friction:
* **Reads like pseudocode:** Clean, readable, intuitive.
* **Explicit scoping:** Blocks open with a colon `:` and close with `end`.
* **Zero garbage collector:** Memory is managed deterministically via lexical scope and compile-time move semantics.
* **Native ternary logic:** Clean 3-state logic (`Trit`) for heuristics, sensor inputs, and uncertain states.

---

## 2. Syntax Fundamentals & Scoping

### 2.1 The Two Invariants
1. **No structural curly braces (`{}`)**: Curly braces are prohibited for scopes, blocks, or format strings.
2. **No semicolons (`;`)**: Statements end naturally with a newline (`\n`).

### 2.2 Colons and `end`
Every code block begins with `:` and concludes with `end`:

```rpl
fn greet(name: String):
    print("Hello, $name!")
end
```

### 2.3 Indentation Freedom
While 4-space indentation is recommended for visual neatness, the compiler does **not** enforce indentation rules. Because scopes are strictly delimited by `:` and `end`, indentation is purely stylistic:

```rpl
// Clean indentation:
if score > 50:
    print("Passed")
end

// Zero indentation (equally valid and parses identically):
if score > 50:
print("Passed")
end
```

### 2.4 Comments
```rpl
// This is a single-line comment

/*
  This is a multi-line
  block comment.
*/
```

---

## 3. Variables and Mutability

### 3.1 Immutable by Default (`let`)
Variables defined with `let` cannot be modified after initial assignment:

```rpl
let speed_limit = 90
// speed_limit = 100  <-- Compile-time error: Cannot mutate immutable variable
```

### 3.2 Mutable Variables (`mut` or `let mut`)
To reassign a variable, declare it with `mut`:

```rpl
let mut counter = 0
counter = counter + 1
```

### 3.3 Explicit Type Annotations
RPL provides type inference, but you can explicitly specify the type:

```rpl
let port: Int = 8080
let hostname: String = "localhost"
let is_active: Bool = true
```

### 3.4 Move Semantics
Passing an owned value into a function transfers ownership. Accessing an identifier after it has been moved produces a compile-time error:

```rpl
let data = "Important Record"
archive(data)
// print(data)  <-- Compile-time error: Use after move
```

---

## 4. Primitive Data Types

| Type | Description | Literals / Examples |
| :--- | :--- | :--- |
| `Int` | 64-bit signed integer | `42`, `-10`, `1_000_000` |
| `Float` | 64-bit IEEE-754 floating point | `3.14159`, `-0.05`, `1.0e-3` |
| `Bool` | Standard 2-state boolean | `true`, `false` |
| `Trit` | 3-state Kleene ternary logic | `true`, `false`, `unknown` |
| `String`| UTF-8 string | `"Hello, world!"` |
| `Void` | Unit / empty return type | Implicit when function returns nothing |

---

## 5. First-Class Ternary Logic (`Trit`)

In many real-world systems (sensor telemetry, heuristics, network polling), states can be uncertain, pending, or missing. RPL provides `Trit` as a native first-class type implementing Kleene logic:

* `true` (+1: Confirmed true)
* `unknown` (0: Undetermined, pending, or missing)
* `false` (-1: Confirmed false)

### 5.1 Truth Tables

| A | B | `A and B` | `A or B` |
| :---: | :---: | :---: | :---: |
| `true` | `true` | `true` | `true` |
| `true` | `unknown` | `unknown` | `true` |
| `true` | `false` | `false` | `true` |
| `unknown` | `unknown` | `unknown` | `unknown` |
| `unknown` | `false` | `false` | `unknown` |
| `false` | `false` | `false` | `false` |

| A | `not A` |
| :---: | :---: |
| `true` | `false` |
| `unknown` | `unknown` |
| `false` | `true` |

### 5.2 Exhaustive Match Verification
When pattern matching on a `Trit`, the compiler **requires** you to handle all three cases:

```rpl
let connection_state: Trit = check_satellite_link()

match connection_state:
    case true:
        print("Link established and verified.")
    case false:
        print("Link failed.")
    case unknown:
        print("Awaiting signal handshake...")
end
// Omitting 'case unknown:' triggers a compile-time NonExhaustiveMatch error!
```

---

## 6. Expressions, Operators & Strings

### 6.1 Arithmetic & Comparison
* Arithmetic: `+`, `-`, `*`, `/`, `%`
* Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
* Logical: `and`, `or`, `not`

### 6.2 The Pipe Operator (`|>`)
The pipe operator passes the result of the left-hand expression as the first argument to the right-hand function, enabling clean data-processing chains:

```rpl
fn double(x: Int) -> Int:
    return x * 2
end

fn increment(x: Int) -> Int:
    return x + 1
end

// Equivalent to: increment(double(5))
let result = 5 |> double() |> increment()
// result is 11
```

### 6.3 String Interpolation
Embed variables directly with `$var` or expressions with `$(expr)`:

```rpl
let item = "Gadget"
let price = 40.0
let tax_rate = 0.20

let summary = "Item $item costs $(price * (1.0 + tax_rate)) EUR with tax."
```

---

## 7. Control Flow

### 7.1 `if`, `else if`, `else`
```rpl
if score >= 90:
    print("Grade: A")
else if score >= 80:
    print("Grade: B")
else:
    print("Retake required")
end
```

### 7.2 `while` Loop
```rpl
let mut i = 0
while i < 10:
    print("Count: $i")
    i = i + 1
end
```

### 7.3 `for` Loop and Ranges
Iterate through a range (`start..end`) or collection:

```rpl
for i in 1..5:
    print("Index: $i")
end

let fruits = ["Apple", "Banana", "Cherry"]
for fruit in fruits:
    print("Fruit: $fruit")
end
```

### 7.4 `match` Control Structure
```rpl
let code = 200

match code:
    case 200:
        print("OK")
    case 404:
        print("Not Found")
    case _:
        print("Unknown Status")
end
```

---

## 8. Functions & Lambdas

### 8.1 Function Declaration
Functions are declared using `fn`. Return types are indicated with `->`:

```rpl
fn multiply(a: Int, b: Int) -> Int:
    return a * b
end

// Functions without a return type return Void:
fn log_info(msg: String):
    println("[INFO] $msg")
end
```

### 8.2 Anonymous Functions (Lambdas)
Lambdas use the concise `param => expression` syntax:

```rpl
let nums = [1, 2, 3, 4, 5]
let doubled = nums.map(x => x * 2)
let evens = nums.filter(x => x % 2 == 0)
```

---

## 9. Custom Types & Collections

### 9.1 Data Structures (`type`)
Custom records are declared using the `type` keyword:

```rpl
type ServerConfig:
    host: String
    port: Int
    ssl_enabled: Bool
    timeout: Float
end

// Instantiation (call-style syntax):
let config = ServerConfig(
    host: "127.0.0.1",
    port: 8080,
    ssl_enabled: true,
    timeout: 30.0
)

// Instantiation (block-style pseudocode syntax):
let local_server = ServerConfig:
    host: "0.0.0.0"
    port: 3000
    ssl_enabled: false
    timeout: 10.0
end

// Accessing fields:
let target_port = config.port
```

### 9.2 Lists (`List[T]`)
```rpl
let numbers = [10, 20, 30, 40]
let first = numbers[0]
```

### 9.3 Maps (`Map[K, V]`)
```rpl
let mut capitals = Map[String, String]()
capitals["Estonia"] = "Tallinn"
capitals["Finland"] = "Helsinki"

let city = capitals.get("Estonia", "Unknown")
```

---

## 10. Concurrency Constructs

RPL incorporates structured concurrency directly into the language syntax:

### 10.1 `spawn:` Block
Executes a task in a lightweight green thread:

```rpl
spawn:
    perform_background_backup()
end
```

### 10.2 `parallel for`
Distributes loop iterations across worker threads:

```rpl
parallel for chunk in data_chunks:
    process_chunk(chunk)
end
```

### 10.3 Channels (`Channel[T]`)
Safe, typed message passing between concurrent tasks:

```rpl
let channel = Channel[Int]()

spawn:
    for i in 1..5:
        channel.send(i * 10)
    end
    channel.close()
end

for val in channel:
    print("Received value: $val")
end
```

---

## 11. Idiomatic Examples

### Example: Sensor Telemetry Decision Matrix

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

fn main():
    let telemetry = SensorReport(
        device_id: "TURBINE-04",
        temperature_nominal: true,
        pressure_valve_open: unknown,
        voltage_stable: true
    )

    let system_integrity: Trit = evaluate_telemetry(telemetry)

    match system_integrity:
        case true:
            print("System operational: All parameters verified.")
        case false:
            print("System emergency: Immediate safety shutdown!")
        case unknown:
            print("Telemetry warning: Incomplete sensor data, retrying probe.")
    end
end
```

---

## 12. Tooling & CLI Execution

RPL provides a unified, cross-platform CLI tool named `rpl`:

### 12.1 Interactive Execution (`rpl run`)
By default, `rpl run` compiles and executes code directly in memory using the native Cranelift JIT engine without emitting intermediate files:

```bash
# Execute instantly via in-memory Cranelift JIT (from repository root or with rpl in PATH):
rpl run examples/reaktor.rpl
```

* **PowerShell / Windows Terminal (local folder execution):**
  ```powershell
  .\rpl.exe run .\reaktor.rpl
  ```
* **Linux / macOS (local folder execution):**
  ```bash
  ./rpl run reaktor.rpl
  ```

To compile and run via the portable C99 pipeline:
```bash
rpl run --via-c examples/reaktor.rpl
```

### 12.2 Static Validation (`rpl check`)
To verify syntax and type exhaustiveness without invoking code generation:

```bash
rpl check examples/reaktor.rpl
# Output: [+ + ?] Check passed: examples/reaktor.rpl
```

### 12.3 Compiling Standalone Binaries (`rpl build`)
Compile directly into an optimized executable or inspect the generated C99 source code:

```bash
# Compile native standalone binary (.exe on Windows, ELF on Linux):
rpl build examples/reaktor.rpl -o reaktor.exe

# Emit clean, human-readable C99 source:
rpl build examples/reaktor.rpl --emit-c -o reaktor.c
```

### 12.4 Trinary Diagnostic Feedback
In alignment with RPL's ternary logic (`true`, `false`, `unknown`), compiler stages report diagnostic vectors using Trits:
```text
[Syntax/Parser . Typechecker . Codegen]
```
* `[+ + +]`: Complete success (e.g. `rpl build`, `rpl run`).
* `[+ + ?]`: Static validation passed, codegen bypassed (`rpl check`).
* `[+ - -]`: Typecheck failure (syntax valid, semantic analysis failed).
* `[- - -]`: Lexer/Parser syntax failure (grammar error, compilation halted).

---

## 13. Two-Tier Input and File I/O (Convenience & Streams)

RPL unites textbook pseudocode readability with true systems programming capabilities through a native, zero-dependency **two-tier I/O model**:

### 13.1 Layer 1: Zero-Ceremony Pseudocode Convenience (Atomic)
For quick scripts, data processing, and rapid prototyping, Layer 1 operations open, execute, and immediately close files in a single atomic step without manual handle lifecycle management:

- `input() -> String`: Reads a line of text from standard input (`stdin`), stripping any trailing `\r` or `\n`.
- `read_file(path: String) -> String`: Reads the entire contents of a file into memory and closes the file handle immediately. If the file cannot be opened, it returns an empty string `""` gracefully.
- `write_file(path: String, content: String) -> Bool`: Overwrites (or creates) the target file with the specified content and immediately closes it. Returns `true` on success, `false` on failure.
- `append_file(path: String, content: String) -> Bool`: Appends the specified content to the end of the target file and immediately closes it. Returns `true` on success, `false` on failure.

#### Pipeline Composition with Layer 1:
Because these functions accept and return pure values, they compose naturally with the pipe operator (`|>`):
```rpl
// Write, append, read, and print in clean pseudocode flow
write_file("status.txt", "INITIALIZED\n")
append_file("status.txt", "HEALTHY\n")

"status.txt" |> read_file |> println
```

---

### 13.2 Layer 2: Long-Lived System Streams & Handles (Daemons & Servers)
RPL is built for high-performance systems engineering. Network daemons, background telemetry loggers, and system servers cannot reopen files for every log entry. Layer 2 introduces persistent stream handles:

- `File`: Opaque system resource handle backed by a standard stream pointer (`FILE*` in C99, `*mut JitFile` in Cranelift JIT).
- `open_file(path: String, mode: String) -> File`: Opens a stream in the requested mode:
  - `"r"`: Read mode.
  - `"w"`: Write/truncate mode.
  - `"a"`: Append mode (ideal for persistent logging).
- `read_line(file: File) -> String`: Reads the next line from the open file handle without closing it.
- `write_line(file: File, line: String) -> Bool`: Writes a line of text followed by a newline character to the stream and flushes the buffer (`fflush`), keeping the handle open.
- `close_file(file: File)`: Flushes and safely closes the open file stream.

```rpl
// Long-lived telemetry logger
let log = open_file("daemon.log", "a")

write_line(log, "[INFO] Daemon started")
write_line(log, "[DEBUG] Listening on port 8080")

// Explicitly close the file when terminating
close_file(log)
```

---

### 13.3 Affine Ownership & Static Use-After-Close Protection
Resource cleanup is statically verified by the RPL typechecker using **affine move semantics**. Calling `close_file(f)` (or using the pipe syntax `f |> close_file`) consumes ownership of the `File` handle:

```rpl
let f = open_file("audit.log", "w")
write_line(f, "System boot")

// close_file moves ownership of handle 'f'
close_file(f)

// COMPILE ERROR: Static verification rejects use-after-close!
write_line(f, "Attempting write after close")
// => [+ - -] Type errors: Use of moved value 'f'
```

This compile-time invariant prevents:
1. Double-close vulnerabilities (`double free` on file descriptors).
2. Dangling file handle operations.
3. Silent data loss from writing to invalidated streams.

---

*For formal grammar specifications, refer to [PROJECT_SPEC.md](PROJECT_SPEC.md).*  
*For compiler codebase architecture, refer to [CODE_MAP.md](CODE_MAP.md).*

