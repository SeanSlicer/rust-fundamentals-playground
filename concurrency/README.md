# concurrency

Threads, channels, and shared state with the standard library only —
no async, no external crates. (Async lives in `async-basics`.)

| Module | Topic | Key takeaway |
|---|---|---|
| `threads` | spawn/join, `move`, scoped threads | a spawned closure must own (or provably outlive-borrow) its data |
| `channels` | mpsc, fan-in, message enums | sending moves ownership — nothing left to race on |
| `shared_state` | Arc, Mutex, RwLock | the borrow rules, enforced at runtime by locks |

The mental model that ties this crate to everything before it: `Send`
and `Sync` are ordinary traits, and the borrow checker rules you
learned in `fundamentals/borrowing` are the same rules a Mutex
enforces dynamically. Rust concurrency is ownership, again.

Decision guide:

- Transfer data between threads → channel (`channels.rs`)
- Share data between threads → `Arc<Mutex<T>>` (`shared_state.rs`)
- Parallelism bounded by one function → scoped threads (`threads.rs`)

```sh
cargo test -p concurrency
```

All tests are deterministic: they assert on joined results and sorted
collections, never on thread scheduling order. Writing concurrency
tests that way is itself one of the lessons.
