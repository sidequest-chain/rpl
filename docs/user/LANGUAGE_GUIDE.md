# The RPL Language Guide (docs/user/LANGUAGE_GUIDE.md)

> **Document Role:** Practical Developer Tutorial & Hands-on User Manual  
> **Target Audience:** Human Programmers & End-User Developers (**Writing Working `.rpl` Code**)  
> **Active Milestone Authority:** 0.2 "Tohtlane" (Active patch: see [VERSION](../../VERSION))  
> **Compiler Ground Truth:** Every code sample in Chapters 1–12 of this guide is **100% verified** to compile and run in the active `0.2` compiler binary (`rpl run` / `rpl build`). Future language specifications and upcoming designs are consolidated in [Chapter 13 (Roadmap Preview)](#13-language-evolution--roadmap-preview-phase-3--4).  
> **Internal Mechanics Reference:** For compiler module architecture, consult [docs/agent/CODE_MAP.md](../agent/CODE_MAP.md) and [docs/agent/COMPILER_CAPABILITIES.md](../agent/COMPILER_CAPABILITIES.md).

Welcome to the **RPL Language Guide**! This handbook is designed as an accessible, hands-on tutorial and reference for programmers writing RPL code. It teaches you how to write clean, high-performance systems pseudocode that compiles directly to native machine code.

---

## Table of Contents

1. [Philosophy: Pseudocode That Executes](#1-philosophy-pseudocode-that-executes)
2. [Syntax Fundamentals & Scoping](#2-syntax-fundamentals--scoping)
3. [Variables, Mutability & Types](#3-variables-mutability--types)
4. [Primitive Data Types](#4-primitive-data-types)
5. [First-Class Ternary Logic (`Trit`)](#5-first-class-ternary-logic-trit)
6. [Expressions, Operators & Strings](#6-expressions-operators--strings)
7. [Control Flow: Conditionals, Loops & Match](#7-control-flow-conditionals-loops--match)
8. [Functions & The Pipe Operator](#8-functions--the-pipe-operator)
9. [Custom Types (`type`)](#9-custom-types-type)
10. [Two-Tier Input / Output System (Convenience & Streams)](#10-two-tier-input--output-system-convenience--streams)
11. [Idiomatic Working Programs](#11-idiomatic-working-programs)
12. [Tooling & CLI Execution (`rpl run`, `build`, `check`, `lsp`)](#12-tooling--cli-execution-rpl-run-build-check-lsp)
13. [Language Evolution & Roadmap Preview (Phase 3 & 4)](#13-language-evolution--roadmap-preview-phase-3--4)

---

## 1. Philosophy: Pseudocode That Executes

Traditional systems programming languages often require extensive punctuation: curly braces `{}`, semicolons `;`, manual lifetime annotations, and boilerplate headers.

**RPL** eliminates this friction:
* **Reads like pseudocode:** Clean, readable, intuitive syntax.
* **Explicit scoping:** Blocks open with a colon `:` and close with `end` (or labeled ends like `end fn`, `end for`, `end match`, `end type`).
* **Zero garbage collector:** Memory is managed deterministically via lexical scope and compile-time move semantics with bare-metal C and Cranelift JIT speed.
* **Native ternary logic:** Built-in 3-state logic (`Trit`) for heuristics, sensor inputs, and uncertain states.

---

## 2. Syntax Fundamentals & Scoping

### 2.1 The Two Invariants
1. **No structural curly braces (`{}`)**: Curly braces are prohibited for scopes, blocks, or format strings.
2. **No semicolons (`;`)**: Statements end naturally with a newline (`\n`).

### 2.2 Colons and `end`
Every code block begins with `:` and concludes with `end`:

```rpl
fn greet(name: String):
    println("Hello, $name!")
end fn
```

You can use plain `end` or self-documenting labeled ends: `end fn`, `end for`, `end if`, `end match`, `end type`.

### 2.3 Indentation Freedom
While 4-space indentation is recommended for visual neatness, the compiler does **not** enforce whitespace rules for scoping. Because scopes are strictly delimited by `:` and `end`, indentation variance does not alter semantics:

```rpl
// Clean indentation:
if score > 50:
    println("Passed")
end if

// Zero indentation (equally valid and parses identically):
if score > 50:
println("Passed")
end if
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

## 3. Variables, Mutability & Types

### 3.1 Immutable by Default (`let`)
Variables defined with `let` cannot be modified after initial assignment:

```rpl
let speed_limit = 90
// speed_limit = 100  <-- Compile-time error: Cannot mutate immutable variable
```

### 3.2 Mutable Variables (`let mut`)
To reassign a variable, declare it with `let mut`:

```rpl
let mut counter = 0
counter = counter + 1
println("Counter: $counter")
```

### 3.3 Explicit Type Annotations
RPL provides type inference, but explicit type annotations are supported:

```rpl
let port: Int = 8080
let temperature: Float = 98.6
let is_active: Bool = true
let status: Trit = unknown
```

---

## 4. Primitive Data Types

The following primitive types are fully supported in the compiler and both execution engines (C99 and Cranelift JIT):

| Type | Description | Literals / Examples |
| :--- | :--- | :--- |
| `Int` | 64-bit signed integer (`i64`) | `42`, `-10`, `1000` |
| `Float` | 64-bit IEEE-754 floating point (`f64`) | `3.14159`, `-0.05`, `100.0` |
| `Bool` | Standard 2-state boolean | `true`, `false` |
| `Trit` | 3-state Kleene ternary logic | `true`, `false`, `unknown` |
| `String`| UTF-8 string with interpolation | `"Hello $name!"`, `"Total: $(p * 2)"` |
| `File` | First-class opaque stream handle | Returned by `open_file(path, mode)` |
| `Void` | Unit / empty return type | Default return type of functions without `->` |

---

## 5. First-Class Ternary Logic (`Trit`)

In systems engineering (telemetry, hardware sensors, network polling, heuristics), states can be uncertain, pending, or unconfirmed. RPL provides `Trit` as a native first-class type implementing Kleene logic:

* `true` (+1: Confirmed true)
* `unknown` (0: Undetermined, pending, or incomplete data)
* `false` (-1: Confirmed false)

### 5.1 Kleene Truth Tables

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
When pattern matching on a `Trit`, the compiler statically **enforces** that all three states (`true`, `false`, `unknown`) are handled:

```rpl
let sensor_ok: Trit = unknown

match sensor_ok:
    case true:
        println("Sensor reading confirmed normal.")
    case false:
        println("Sensor alarm: Critical reading!")
    case unknown:
        println("Telemetry warning: Sensor signal pending, re-scanning...")
end match
// Omitting 'case unknown:' triggers compile-time NonExhaustiveMatch error!
```

---

## 6. Expressions, Operators & Strings

### 6.1 Operators
* **Arithmetic:** `+`, `-`, `*`, `/`, `%`
* **Comparison:** `==`, `!=`, `<`, `<=`, `>`, `>=`
* **Logical:** `and`, `or`, `not`
* **Bitwise:** `&`, `|`, `^`, `<<`, `>>`, `~`

### 6.2 String Interpolation
Embed variables directly with `$var` or evaluated sub-expressions with `$(expr)`:

```rpl
let item = "Turbine Core"
let base_temp = 320.0
let delta = 14.5

println("Component $item operating at $(base_temp + delta) C.")
```

---

## 7. Control Flow: Conditionals, Loops & Match

### 7.1 `if`, `else if`, `else`
```rpl
let score = 85

if score >= 90:
    println("Grade: A")
else if score >= 80:
    println("Grade: B")
else:
    println("Retake required")
end if
```

### 7.2 Range Loops (`for in start..end:`)
In milestone 0.2, iterative execution is handled via high-speed, bounded integer range loops:

```rpl
let mut sum = 0
for i in 1..5:
    sum = sum + i
    println("Step $i -> Running sum: $sum")
end for

println("Final sum: $sum")
```

Range loops execute with native register speed without heap allocation or iterator overhead.

### 7.3 Pattern Matching (`match`)
RPL features pattern matching over integers, booleans, Trits, and wildcards (`_`):

```rpl
let http_status = 200

match http_status:
    case 200:
        println("HTTP 200: Success")
    case 404:
        println("HTTP 404: Not Found")
    case 500:
        println("HTTP 500: Internal Server Error")
    case _:
        println("HTTP $http_status: Unhandled Status Code")
end match
```

---

## 8. Functions & The Pipe Operator

### 8.1 Function Declarations (`fn`)
Top-level functions are declared using `fn`. Parameter type annotations are mandatory. Return types are indicated with `->`:

```rpl
fn calculate_flux(temp: Float, pressure: Float) -> Float:
    return (temp * 1.414) + pressure
end fn

fn log_metric(name: String, val: Float):
    println("[METRIC] $name = $val")
end fn
```

Functions without an explicit `-> ReturnType` default to `Void`.

### 8.2 The Pipe Operator (`|>`)
The pipe operator passes the result of the left-hand expression as the first argument to the right-hand function:

```rpl
fn double_val(x: Int) -> Int:
    return x * 2
end fn

fn add_ten(x: Int) -> Int:
    return x + 10
end fn

// Equivalent to: add_ten(double_val(5))
let result = 5 |> double_val |> add_ten
println("Result: $result") // Prints: 20
```

---

## 9. Custom Types (`type`)

### 9.1 Declaring Composite Structs
Custom records are declared using the `type` keyword:

```rpl
type CoreReactor:
    id: String
    temperature: Float
    pressure_nominal: Trit
    cooling_active: Trit
end type
```

### 9.2 Struct Block Instantiation
RPL supports readable pseudocode block-style struct instantiation:

```rpl
let r1 = CoreReactor:
    id: "UNIT-7"
    temperature: 412.8
    pressure_nominal: true
    cooling_active: unknown
end CoreReactor
```

Standard call-style constructor instantiation is also supported:
```rpl
let r2 = CoreReactor("UNIT-8", 295.0, true, true)
```

### 9.3 Accessing and Modifying Fields
```rpl
println("Reactor $(r1.id) Temp: $(r1.temperature)")

// Mutating a field on a mutable struct:
let mut r_active = r1
r_active.temperature = 425.0
println("Updated temp: $(r_active.temperature)")
```

---

## 10. Two-Tier Input / Output System (Convenience & Streams)

RPL unites textbook pseudocode readability with true systems programming capabilities through a native **two-tier I/O model**:

### 10.1 Layer 1: Zero-Ceremony Pseudocode Convenience (Atomic)
For quick scripts and data processing, Layer 1 operations open, execute, and immediately close files in a single atomic step without manual handle lifecycle management:

- `input() -> String`: Reads a line of text from standard input (`stdin`), stripping trailing newlines.
- `read_file(path: String) -> String`: Reads an entire file into memory and closes the file immediately. Returns `""` gracefully on error.
- `write_file(path: String, content: String) -> Bool`: Overwrites or creates the target file with content and closes it. Returns `true` on success.
- `append_file(path: String, content: String) -> Bool`: Appends content to the target file and closes it. Returns `true` on success.

```rpl
// Write and append in a clean pseudocode flow
write_file("status.txt", "INITIALIZED\n")
append_file("status.txt", "TELEMETRY NOMINAL\n")

let content = read_file("status.txt")
println("File Content:\n$content")
```

---

### 10.2 Layer 2: Long-Lived System Streams & Handles (Daemons & Servers)
System servers, continuous background loggers, and telemetry daemons cannot afford to reopen files for every log entry. Layer 2 introduces persistent stream handles:

- `File`: Opaque system resource handle (`FILE*` in C99, `*mut JitFile` in Cranelift JIT).
- `open_file(path: String, mode: String) -> File`: Opens a stream in mode `"r"`, `"w"`, or `"a"`.
- `read_line(file: File) -> String`: Reads the next line from the open file handle without closing it.
- `write_line(file: File, line: String) -> Bool`: Writes a line of text followed by newline to the stream and flushes the buffer (`fflush`).
- `close_file(file: File)`: Flushes and safely closes the open file stream.

```rpl
let log = open_file("daemon.log", "a")

write_line(log, "[INFO] Telemetry service daemon started")
write_line(log, "[DEBUG] Sensors connected: 4 active units")
write_line(log, "[INFO] Telemetry daemon shutting down cleanly")

close_file(log)
```

---

### 10.3 Affine Move Ownership: Compile-Time Use-After-Close Protection
Resource lifecycle is statically verified by the RPL typechecker using **affine move semantics**. Calling `close_file(f)` consumes ownership of the `File` handle:

```rpl
let f = open_file("audit.log", "w")
write_line(f, "System boot")

// close_file moves ownership of handle 'f'
close_file(f)

// COMPILE ERROR: The typechecker statically rejects use-after-close!
// write_line(f, "Attempting write after close")
// => [+ - -] Type errors: Use of moved value 'f'
```

This prevents double-close bugs, dangling file descriptors, and silent data loss at compile time.

---

## 11. Idiomatic Working Programs

Both of these programs are complete, self-contained, and verified to compile and run in milestone 0.2:

### Program 1: Nuclear Reactor Diagnostic Telemetry (`examples/reaktor.rpl`)

```rpl
type Reactor:
    id: String
    temperature: Float
    pressure_nominal: Trit
    cooling_active: Trit
end Reactor

fn calculate_safety(r: Reactor) -> Trit:
    // Kleene logic:
    // If pressure is false, system is compromised regardless of cooling
    // If cooling is unknown, safety state is indeterminate (unknown)
    return r.pressure_nominal and r.cooling_active
end fn

fn report(reactor: Reactor):
    let status = calculate_safety(reactor)
    
    println("==========================================")
    println("Reactor Report: $reactor.id")
    println("Temperature: $reactor.temperature C")
    println("Pressure Nominal: $reactor.pressure_nominal")
    println("Cooling: $reactor.cooling_active")
    println("------------------------------------------")
    
    match status:
        case true:
            println("AUTOMATION: System is 100% stable.")
        case false:
            println("ALARM: Critical failure! Trigger emergency shutdown!")
        case unknown:
            println("WARNING: Incomplete telemetry! Requires engineer inspection.")
    end match
    println("==========================================\n")
end fn

// Run diagnostic cycles
for cycle in 1..3:
    println("Starting diagnostic cycle: $cycle")
    if cycle == 1:
        let r1 = Reactor:
            id: "R-01-A"
            temperature: 312.5
            pressure_nominal: true
            cooling_active: true
        end Reactor
        report(r1)
    else if cycle == 2:
        let r2 = Reactor:
            id: "R-02-B"
            temperature: 489.1
            pressure_nominal: unknown
            cooling_active: true
        end Reactor
        report(r2)
    else:
        let r3 = Reactor:
            id: "R-03-C"
            temperature: 720.0
            pressure_nominal: false
            cooling_active: unknown
        end Reactor
        report(r3)
    end if
end for
```

---

### Program 2: Persistent Daemon Logger (`examples/daemon_logger.rpl`)

```rpl
fn run_logger_daemon():
    println("Starting Telemetry Daemon Logger...")
    let log_path = "system_daemon.log"

    let file = open_file(log_path, "w")
    write_line(file, "[INFO] Telemetry service daemon started")
    write_line(file, "[DEBUG] Sensors connected: 4 active units")

    for tick in 1..3:
        write_line(file, "[TRACE] Heartbeat pulse tick #$tick - All metrics nominal")
    end for

    write_line(file, "[INFO] Telemetry daemon shutting down cleanly")
    close_file(file)

    println("Daemon log completed and closed safely.")
    println("--- Final Log Dump ---")
    let final_content = read_file(log_path)
    println("$final_content")
end fn

run_logger_daemon()
```

---

## 12. Tooling & CLI Execution (`rpl run`, `build`, `check`, `lsp`)

The unified CLI binary `rpl` provides fast developer workflows:

### 12.1 Direct In-Memory JIT Execution (`rpl run`)
Compiles and executes code directly in memory via Cranelift with sub-10ms latency:

```powershell
# Windows
.\target\debug\rpl.exe run examples/reaktor.rpl

# Linux / macOS
./target/debug/rpl run examples/reaktor.rpl
```

### 12.2 Static Typecheck & Validation (`rpl check`)
Validates grammar, types, Trit exhaustiveness, and affine move semantics without code generation:

```bash
rpl check examples/reaktor.rpl
# Output: [+ + ?] Check passed: examples/reaktor.rpl
```

### 12.3 Standalone Binary Compilation (`rpl build`)
Transpiles through the C99 backend and invokes the host C compiler (GCC/Clang/MSVC) to produce a standalone executable:

```powershell
rpl build examples/reaktor.rpl -o reaktor.exe
```

Or inspect the self-contained C99 source code directly:
```powershell
rpl build --emit-c examples/reaktor.rpl
```

### 12.4 Language Server Protocol (`rpl lsp`)
RPL includes a zero-dependency Language Server implementing the LSP standard. To configure Zed, VS Code, or Antigravity IDE, see [docs/user/IDE_SETUP.md](IDE_SETUP.md).

---

## 13. Language Evolution & Roadmap Preview (Phase 3 & 4)

> [!IMPORTANT]
> **ROADMAP PREVIEW NOTICE:**  
> The constructs in this section represent RPL's planned architectural evolution described in [docs/spec/PROJECT_SPEC.md](../spec/PROJECT_SPEC.md) and [docs/spec/ROADMAP.md](../spec/ROADMAP.md). They are **not yet executable** in milestone 0.2 and will be unlocked in upcoming milestone releases.

### 13.1 `while` Loop Construct (Phase 2 Extension / Phase 3)
Planned syntax for indefinite conditional iteration:

```rpl
// Future milestone target:
let mut i = 0
while i < 10:
    println("Count: $i")
    i = i + 1
end while
```
*Current 0.2 Workaround:* Use bounded `for i in 1..limit:` loops with `if` conditionals.

### 13.2 Dynamic Collections & Functional Methods (Milestone 0.4 "Tulihänd")
Planned standard library collection types and functional methods:

```rpl
// Future milestone target:
let nums = [1, 2, 3, 4, 5]
let doubled = nums.map(x => x * 2)
let evens = nums.filter(x => x % 2 == 0)

let mut capitals = Map[String, String]()
capitals["Estonia"] = "Tallinn"
let city = capitals.get("Estonia", "Unknown")
```
*Current 0.2 Workaround:* Use fixed record structures (`type`) and top-level pipeline functions (`|>`).

### 13.3 Structured Concurrency & Channels (Milestone 0.3 "Kratt")
Planned green-thread task spawning and lock-free typed channels:

```rpl
// Future milestone target:
let channel = Channel[Int]()

spawn:
    for i in 1..5:
        channel.send(i * 10)
    end for
    channel.close()
end spawn

for val in channel:
    println("Received: $val")
end for
```
*Current 0.2 Workaround:* Milestone 0.2 executes structured blocks sequentially. True OS-level thread dispatch is slated for milestone 0.3.

### 13.4 Algebraic `Result[T, E]` and Error Constructors
Planned algebraic error handling to complement Trit logic:

```rpl
// Future milestone target:
fn divide(a: Float, b: Float) -> Result[Float, String]:
    if b == 0.0:
        return Error("Division by zero")
    end if
    return Ok(a / b)
end fn
```
*Current 0.2 Workaround:* Use `Trit` (`unknown` for failure/missing state) or status fields on custom structs.

---

*For formal grammar specifications, refer to [docs/spec/PROJECT_SPEC.md](../spec/PROJECT_SPEC.md).*  
*For compiler codebase architecture, refer to [docs/agent/CODE_MAP.md](../agent/CODE_MAP.md).*
