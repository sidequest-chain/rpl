# RPL Structured Concurrency & Channel Specification (docs/spec/CONCURRENCY_SPEC.md)

> **Document Role:** Formal Specification & Long-Term Language Architecture (Vision / RFC)  
> **Target Audience:** Language Designers, Compiler Architects  
> **Warning for LLM Agents:** Contains language features planned for future milestones. Do NOT assume constructs documented here are currently implemented in the compiler backend unless verified in [docs/agent/COMPILER_CAPABILITIES.md](../agent/COMPILER_CAPABILITIES.md).  
> **Milestone Target:** Milestone 0.3 "Kratt"  
> **Active Milestone:** 0.2 "Tohtlane" (Active patch: see [VERSION](../../VERSION))  
> **Classification:** RFC / Language Extension Specification  
> **Status:** APPROVED RFC & DESIGN TARGET  

---

## 1. Executive Summary & Architectural Philosophy

The Running Pseudo Language (RPL) is designed to execute clean, readable pseudocode with bare-metal speed while strictly eliminating data races, deadlocks, and unpredictable latency without relying on a tracing garbage collector (GC). 

Milestone **0.3 "Kratt"** introduces first-class **Structured Concurrency** and **Lock-Free Typed Channels**. Concurrency in RPL is founded on four immutable principles:

1. **Structured Concurrency by Default:** Concurrent tasks are lexically bounded. A lexical scope that initiates child tasks via `spawn:` or `parallel for` acts as an automatic join barrier—it cannot exit until all spawned child tasks complete, eliminating orphan thread leaks and background zombie tasks.
2. **Shared-Nothing Memory Isolation:** Tasks communicate exclusively via message passing across typed channels (`Channel[T]`). Memory is either exclusively owned by a single task or immutably shared. Mutable state is never shared across task boundaries.
3. **Deterministic Scope-Based Memory Safety (Zero-GC):** When values are sent through a channel, ownership is transferred (*affine move semantics*). The sender relinquishes ownership at compile time, guaranteeing zero-copy or clean move semantics without reference counting overhead or stop-the-world garbage collection pauses.
4. **Cooperative M:N Task Scheduling:** Green threads (lightweight tasks) are scheduled cooperatively across a fixed-size pool of OS worker threads, allowing tens of thousands of concurrent tasks to execute with minimal stack and memory overhead.

---

## 2. Grammar & Syntactic Specifications

### 2.1 The `spawn:` Task Block
The `spawn:` keyword introduces a new asynchronous task executed cooperatively by the runtime scheduler.

```rpl
spawn:
    // Task body executed concurrently
    let result = compute_work()
    results_channel.send(result)
end spawn
```

#### Syntax Invariants:
* **Opening Delimiter:** The block must open with `:` and close with `end` or the matching label `end spawn`.
* **Nesting Depth Rule ($\ge 3$):** If the `spawn:` block resides at nesting depth 3 or greater, the parser strictly rejects bare `end` with `ParserError::AmbiguousBlockEnd`. The closing delimiter must be explicitly labeled `end spawn`.
* **Lexical Join Barrier:** The enclosing lexical scope (e.g. parent function or block) implicitly joins and awaits all enclosed `spawn:` tasks before executing subsequent statements or returning, unless an explicit detachment primitive is invoked.

### 2.2 Multi-Core Data Parallelism (`parallel for`)
The `parallel for` construct distributes iterations of an enumerable collection or numeric range across worker threads:

```rpl
parallel for path in targets:
    let info = inspect_file(path)
    results_channel.send(info)
end for
```

#### Syntax Invariants:
* **Block Label:** Closes with `end for` or `end`.
* **Captured Variables:** Variables captured from the outer scope must be immutable (`let`). Attempting to mutate an outer variable inside a `parallel for` body triggers compile-time rejection:
  ```text
  TypeError: Cannot mutate variable 'counter' from within a parallel for iteration block.
  ```

---

## 3. Typed Lock-Free Channels (`Channel[T]`)

Communication between concurrent tasks is mediated entirely by typed channels.

### 3.1 Instantiation & Type Signature
Channels are parameterized by the message payload type `T`:

```rpl
// Unbounded channel:
let results_channel = Channel[LogMetadata]()

// Bounded channel with backpressure capacity:
let bounded_pipe = Channel[Int](capacity: 128)
```

### 3.2 Channel Operations

| Operation | Method Signature | Operational Semantics |
| :--- | :--- | :--- |
| **Send** | `channel.send(value: T)` | Pushes `value` into the channel. If bounded and full, blocks cooperatively until capacity is available. Transfers ownership of `value`. |
| **Receive** | `channel.recv() -> T` | Awaits and returns the next message. Blocks cooperatively if empty. |
| **Close** | `channel.close()` | Closes the channel. No further values can be sent (`channel.send` will error). Remaining buffered values can still be drained. |
| **Status** | `channel.is_closed() -> Bool` | Non-blocking check returning `true` if the channel has been closed. |

### 3.3 Channel Iteration (Stream Draining)
Channels integrate natively with standard RPL `for` loops:

```rpl
for metadata in results_channel:
    println("Processed: $metadata.path ($metadata.line_count lines)")
    total_lines = total_lines + metadata.line_count
end for
```

#### Draining Semantics:
* The `for in` loop continuously consumes values from `results_channel`.
* If the channel is empty but not closed, the task cooperatively yields execution until a new value is sent.
* Once the channel is closed and all buffered elements are drained, the loop terminates cleanly and control passes to the following statement.

---

## 4. Affine Move Semantics Across Channels

To guarantee data-race freedom without a garbage collector or runtime locks, RPL enforces compile-time ownership verification for channel operations:

1. **Move on Send:**
   Sending a value transfers ownership to the channel buffer:
   ```rpl
   let file_info = inspect_file(path)
   results_channel.send(file_info)
   // Use of file_info here is strictly rejected at compile time:
   // TypeError: Use of moved value 'file_info'
   ```
2. **Channel Handle Passing:**
   A channel handle (`Channel[T]`) is an opaque, thread-safe reference type. Multiple concurrent tasks may hold cloneable send/receive handles to the underlying queue without data races.

---

## 5. Execution Backends & Runtime Bridge

### 5.1 Portable C99 Backend (`rpl_codegen_c`)
In C99 transpilation mode (`rpl build`), the compiler emits calls to the RPL C Concurrency Runtime (`rpl_concurrency.h`):
* **Task Pool:** Fixed worker pool utilizing cross-platform OS threads (`tinycthread` or standard Windows/POSIX threads).
* **Queue Implementation:** Lock-free Michael & Scott or bounded Vyukov ring buffer MPMC (Multi-Producer Multi-Consumer) queues.
* **Join Barriers:** Scope exit emits `rpl_task_group_wait(&__scope_tasks);` ensuring zero orphan task escapes.

### 5.2 Cranelift In-Memory JIT Backend (`rpl_codegen_cranelift`)
In JIT execution mode (`rpl run`), the runtime exposes ABI function pointers registered in the Cranelift namespace:
* `rpl_jit_channel_new(elem_size: usize, capacity: usize) -> *mut JitChannel`
* `rpl_jit_channel_send(chan: *mut JitChannel, data: *const u8)`
* `rpl_jit_channel_recv(chan: *mut JitChannel, out_data: *mut u8) -> bool`
* `rpl_jit_channel_close(chan: *mut JitChannel)`
* `rpl_jit_spawn(task_fn: extern "C" fn(*mut u8), env: *mut u8)`

---

## 6. Official Canonical Grammar Target (`tests/fixtures/parallel.rpl`)

The following program serves as the official test fixture and syntactic baseline for Phase 3 "Kratt" concurrency:

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
        println("Processed: $metadata.path ($metadata.line_count lines)")
        total_lines = total_lines + metadata.line_count
    end

    println("Parallel inspection complete. Total log lines: $total_lines")
end

run_pipeline()
```

---

## 7. Cross-Reference Index

* **Active Ground Truth:** [docs/agent/COMPILER_CAPABILITIES.md](../agent/COMPILER_CAPABILITIES.md)
* **Codebase Topography & Symbol Map:** [docs/agent/CODE_MAP.md](../agent/CODE_MAP.md)
* **Phased Milestone Roadmap:** [docs/agent/ROADMAP.md](../agent/ROADMAP.md)
* **Formal Language Specification:** [docs/spec/PROJECT_SPEC.md](PROJECT_SPEC.md)
* **End-User Language Guide:** [docs/guide/LANGUAGE_GUIDE.md](../guide/LANGUAGE_GUIDE.md)
