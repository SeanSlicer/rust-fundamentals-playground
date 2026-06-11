# rust-fundamentals-playground

A Rust learning workspace built around real, compiling, tested code
instead of toy snippets. Every concept lives in a module you can read
top to bottom, run, break, and fix. Common mistakes are part of the
material: `compile_fail` doctests show code the compiler rejects (and
`cargo test` verifies it still rejects them), `should_panic` tests
document runtime failure modes, and every module ends with exercises.

## Layout

```text
fundamentals/   beginner topics, one module each (start here)
intermediate/   traits, generics, lifetimes, closures, pointers, modules
concurrency/    threads, channels, Arc/Mutex — std only
async-basics/   async/await with Tokio
projects/       six small, complete programs applying all of it
```

## Learning path

1. **`fundamentals`** — read the modules in the order `src/lib.rs`
   declares them: variables → ownership → borrowing → slices →
   functions/control flow → structs → enums → pattern matching →
   error handling → collections → strings → iterators.
2. **`intermediate`** — traits → generics → lifetimes, then closures,
   smart pointers, interior mutability, and the module system.
3. **First projects** — `calc`, then `todo`, then `config-loader`.
   Each applies several fundamentals at once.
4. **Parsing projects** — `csv-parser` (strict, state machine) and
   `log-parser` (lenient, best-effort); comparing the two is the
   lesson.
5. **`concurrency`** — threads, channels, shared state.
6. **`async-basics`** and **`http-client`** — async I/O concepts and
   the protocol underneath every HTTP library.

## Running things

```sh
cargo test                      # entire workspace: ~150 tests
cargo test -p fundamentals      # one crate
cargo test -p fundamentals ownership   # one topic
cargo run -p calc -- "(1 + 2) * -3.5"
cargo run -p todo-cli -- add "learn rust"
cargo run -p log-parser -- projects/log-parser/sample.log
cargo run -p csv-parser -- projects/csv-parser/people.csv
cargo run -p config-loader -- projects/config-loader/app.example.toml
cargo run -p async-basics
cargo run -p http-client        # the only one needing network
```

## Repository map: where each concept lives

| Concept | Where |
|---|---|
| Variables, mutability, shadowing | `fundamentals/src/variables.rs` |
| Ownership, moves, Copy, clone | `fundamentals/src/ownership.rs` |
| Borrowing & references | `fundamentals/src/borrowing.rs` |
| Slices (`&[T]`, `&str`) | `fundamentals/src/slices.rs` |
| Functions & control flow | `fundamentals/src/functions_control_flow.rs` |
| Structs, methods, newtypes | `fundamentals/src/structs.rs` |
| Enums & Option | `fundamentals/src/enums.rs` |
| Pattern matching | `fundamentals/src/pattern_matching.rs` |
| Result, `?`, custom errors | `fundamentals/src/error_handling.rs` |
| Collections (Vec, HashMap...) | `fundamentals/src/collections.rs` |
| Strings & UTF-8 | `fundamentals/src/strings.rs` |
| Iterators (incl. custom) | `fundamentals/src/iterators.rs` |
| Traits, dispatch, operators | `intermediate/src/traits.rs` |
| Generics & bounds | `intermediate/src/generics.rs` |
| Lifetimes | `intermediate/src/lifetimes.rs` |
| Closures (Fn/FnMut/FnOnce) | `intermediate/src/closures.rs` |
| Box, Rc, Weak | `intermediate/src/smart_pointers.rs` |
| Cell, RefCell | `intermediate/src/interior_mutability.rs` |
| Modules & visibility | `intermediate/src/modules_demo.rs` |
| Unit vs integration tests | `fundamentals/tests/learning_checks.rs` |
| Threads, scoped threads | `concurrency/src/threads.rs` |
| Channels (mpsc) | `concurrency/src/channels.rs` |
| Arc, Mutex, RwLock | `concurrency/src/shared_state.rs` |
| async/await, join!, spawn | `async-basics/src/lib.rs` |
| File I/O | `projects/todo/src/lib.rs`, `projects/config-loader/src/lib.rs` |
| Serde (JSON) | `projects/todo/src/lib.rs` |
| Serde (TOML) + layered config | `projects/config-loader/src/lib.rs` |
| Recursive descent parsing | `projects/calc/src/lib.rs` |
| State machines with enums | `projects/csv-parser/src/lib.rs` |
| FromStr, best-effort parsing | `projects/log-parser/src/lib.rs` |
| Raw TCP / HTTP | `projects/http-client/src/lib.rs` |
| Cargo workspaces | `Cargo.toml` (root) |

## Conventions

- Comments explain **why**, not what. If a line needs a "what"
  comment, the line gets rewritten instead.
- Every project splits a testable library (`lib.rs`) from a thin I/O
  binary (`main.rs`).
- Tests are deterministic: no network in tests, no thread-timing
  assertions, paused virtual time for async timing.

## Prerequisites

A recent stable Rust toolchain (`rustup update stable`). Scoped
threads need 1.63+; everything else is comfortably within stable.
