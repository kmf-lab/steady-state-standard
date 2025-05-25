# Steady State Standard

A production-ready actor system demonstrating core concurrent programming patterns with the `steady_state` framework.

## System Architecture

**Data Flow Pipeline**:
Generator → Worker ← Heartbeat → Logger

- **Generator**: Continuous data production with persistent state
- **Heartbeat**: Timing control and lifecycle management
- **Worker**: Batch processing with backpressure handling
- **Logger**: Message consumption and output

## Threading Model

### Actor-Per-Thread Isolation
Each actor runs on its own dedicated thread using `Threading::Spawn`, providing complete memory isolation. Unlike traditional threading models that share memory and require locks, actors communicate exclusively through channels.

**Traditional Threading Problems Eliminated**:
- **Race Conditions**: Impossible since actors don't share memory
- **Deadlocks**: No shared locks to create circular dependencies
- **Data Races**: Each thread owns its state exclusively
- **Manual Synchronization**: Framework handles coordination automatically

### Thread Coordination Patterns

**Cooperative Shutdown**: When any actor calls `request_graph_stop()`, the shutdown signal propagates to all threads. Each actor completes its current work and closes its channels, allowing dependent actors to drain remaining messages before terminating.

**Backpressure Across Threads**: The framework automatically manages flow control between threads. When a receiving actor's channel fills up, sending actors block until space becomes available, preventing system overload without explicit coordination.

**Synchronized Operations**: The `await_for_all!` macro coordinates multiple async conditions across thread boundaries, such as waiting for both periodic timers and channel availability simultaneously.

### Channel-Based Communication

Channels replace traditional shared memory patterns:
- **Type Safety**: Compile-time guarantees about message types
- **Bounded Buffers**: Automatic backpressure when channels fill
- **Clean Shutdown**: Channels can be marked closed to signal completion
- **Zero-Copy**: Messages move between threads without duplication when possible

## Core Features

### Actor State Management
State persists across actor restarts using `SteadyState<T>`, enabling fault tolerance without complex recovery logic.

### Dual-Mode Operation
Same actor code runs in production and test environments through conditional behavior switching.

### Flow Control Primitives
- Coordinate multiple async conditions with `await_for_all!`
- Manage backpressure with `SendSaturation::AwaitForRoom`
- Handle periodic operations with `wait_periodic()`
- Monitor channel states with `wait_vacant()` and `wait_avail()`

### Message Processing Patterns

**Batch Processing**: Workers process data in batches triggered by heartbeat signals, enabling efficient bulk operations while maintaining responsive timing control.

**Continuous Processing**: Generators produce data continuously until shutdown, with automatic flow control preventing buffer overflows.

**Event-Driven Processing**: Loggers respond to incoming messages immediately, demonstrating reactive patterns within the actor framework.

## Testing Framework

### Thread-Safe Testing
The testing framework handles multi-threaded actor coordination automatically. Test scenarios can inject messages, trigger specific actor behaviors, and verify outcomes across multiple threads without manual synchronization.

### Stage Management
Integration tests use stage managers to orchestrate complex multi-actor scenarios, directing specific actors to perform actions and waiting for expected results across the entire system.

## Threading Advantages

| Traditional Approach | Actor Model Approach |
|---------------------|---------------------|
| Shared memory + locks | Isolated memory + messages |
| Manual thread coordination | Framework-managed lifecycle |
| Difficult testing | Built-in test support |
| Race condition debugging | Race conditions impossible |
| Complex shutdown logic | Cooperative shutdown |

## Usage

```bash
cargo run                           # Default: 1s rate, 60 beats
cargo run -- --rate 100 --beats 20 # Fast: 100ms rate, 20 beats
cargo test                          # Run all tests including multi-threaded scenarios
```

## Key Capabilities

- **Thread-per-actor isolation** eliminates concurrency bugs
- **Automatic coordination** across multiple threads
- **Graceful shutdown** propagates through all threads
- **Built-in monitoring** tracks per-thread CPU and performance
- **Crash-resilient state** survives individual thread failures
- **Comprehensive testing** validates multi-threaded behavior
- **Zero-configuration** thread management

This threading model scales from simple utilities to complex distributed systems while maintaining deterministic, race-free behavior across all threads.

