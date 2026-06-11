# async-basics

A first contact with `async`/`.await` using Tokio.

```sh
cargo run -p async-basics    # watch sequential vs concurrent timings
cargo test -p async-basics
```

What's covered, in reading order through `src/lib.rs`:

1. `async fn` returns a lazy Future — nothing runs until awaited
2. Sequential awaits vs `tokio::join!` (the concurrency win)
3. `tokio::spawn` — independent tasks, the async `thread::spawn`
4. Async mpsc channels with backpressure

Worth internalizing early:

- **Rust ships the syntax, not the runtime.** `async`/`.await` are
  language features; the executor (Tokio, async-std, smol...) is a
  library you choose.
- **Never block inside async.** `std::thread::sleep`, heavy
  computation, or blocking I/O inside an async fn stalls every task
  sharing that worker thread. Use `tokio::time::sleep`,
  `spawn_blocking`, or plain threads (`concurrency` crate).
- **Async is for waiting, threads are for working.** I/O-bound with
  high concurrency → async. CPU-bound → threads.

The tests use `#[tokio::test(start_paused = true)]`: the runtime
fakes the clock, so the "this must overlap" assertions are exact and
run instantly instead of being flaky wall-clock measurements.
