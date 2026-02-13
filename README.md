# rrstat — Lightweight Linux Profiler

A minimalist, sampling-based profiler for Linux written in Rust. It uses `ptrace` to capture instruction pointers and `addr2line` for symbol resolution across multiple binaries.

![Profiler Summary Screenshot](https://github.com/Meredan/rrstat/blob/main/Screenshot%20from%202026-02-11%2020-43-05.png)

## Features

- **Sampling Profiler**: Uses statistical sampling to minimize overhead.
- **RIP Capture**: Reliably captures the Instruction Pointer (RIP) using `ptrace` at specified intervals.
- **Shared Library Support**: Automatically resolves symbols in shared libraries (e.g., `libc`, `libm`) by parsing `/proc/[pid]/maps`.
- **Address Translation**: Handles ASLR by calculating relative offsets for PIE (Position Independent Executables) and shared objects.
- **Thread-safe Buffer**: Efficient, lock-free (single-writer/single-reader) Ring Buffer for sample collection.
- **C-Demangling**: Support for demangling C++ and Rust symbols.

## Supported Sampling Events

`rrstat` allows you to profile different aspects of your application by choosing different sampling events via the `--event` flag.

### Hardware Events
*   **`cpu-cycles`** (Default): Measures the actual clock cycles consumed by the CPU.
    *   **Meaning**: Identifies "hot" code paths where the CPU is doing the most work. This is the primary metric for optimizing computation-heavy tasks.
*   **`instructions`**: Counts the number of retired instructions.
    *   **Meaning**: Helps understand the complexity of the code path. Comparing this with `cpu-cycles` can reveal low IPC (Instructions Per Cycle), suggesting stalls.
*   **`cache-misses`**: Measures L1/L2/L3 cache misses.
    *   **Meaning**: High cache misses indicate memory bottlenecks. Improving data locality or reducing pointer chasing can often yield 10x speedups here.

### Software Events & Wait Time
*   **`task-clock`**: A timer that only runs when the task is actively scheduled on a CPU.
    *   **Meaning**: Filters out time when the process is blocked (e.g., waiting for I/O). Useful for analyzing pure compute performance.
*   **`cpu-clock`** / **`wait-time`**: A high-resolution wall-clock timer.
    *   **Meaning**: Profiles the total time spent in a function, **including time spent waiting** for I/O, locks, or network. If a function is high in `cpu-clock` but low in `task-clock`, it is **wait-bound** (I/O-bound).
*   **`context-switches`**: Counts how often the task was switched out.
    *   **Meaning**: High counts usually point to excessive synchronization (lock contention) or frequent small I/O operations.
*   **`page-faults`**: Counts memory page faults.
    *   **Meaning**: Indicates high memory pressure or inefficient memory allocation patterns (e.g., large allocations that aren't reused).

## Implementation Details

- **Collector**: Spawns a background thread that periodically interrupts the target process via `PTRACE_ATTACH`, reads registers, and resumes execution.
- **SymbolResolver**: Caches `addr2line` contexts for all mapped executable files to allow fast, multi-binary resolution.
- **Aggregator**: Processes raw samples into a summary report, folding identical stack/instruction counts and calculating percentages.

## Getting Started

### Prerequisites

- Linux OS
- Rust toolchain
- `gcc` (for test targets)

### Building

```bash
cargo build --release
```

### Usage

To profile a process, you typically need root privileges or `cap_sys_ptrace` because the profiler uses `ptrace` to attach to running PIDs.

```bash
# Profile a specific PID for 5 seconds (default: cpu-cycles)
sudo ./target/release/rrstat --pid <PID> --duration 5000

# Profile for cache misses
sudo ./target/release/rrstat --pid <PID> --event cache-misses

# Profile total wait time vs compute
sudo ./target/release/rrstat --pid <PID> --event wait-time
```

### Testing

The project includes integration tests that verify end-to-end sampling and symbol resolution:

```bash
# Run all tests
cargo test

# Run a specific real-world sampling test (no sudo needed for child processes)
cargo test test_real_collector -- --nocapture
```

## Project Structure

- `src/collector.rs`: Core `ptrace` sampling loop.
- `src/symbols.rs`: High-level symbol resolution with context caching.
- `src/maps.rs`: Helper for parsing `/proc/[pid]/maps`.
- `src/aggregator.rs`: Statistics calculation and folding.
- `src/report.rs`: Formatted reporting logic.
- `src/main.rs`: Application entry point and signal handling.
