# fundamentals

Beginner-level Rust, one module per topic. Work through the modules in
the order they appear in `src/lib.rs` — each builds on the previous.

| Module | Topic | Key takeaway |
|---|---|---|
| `variables` | bindings, `mut`, shadowing, constants | immutable by default; shadowing can change type |
| `ownership` | moves, clones, `Copy` | every value has one owner; moves invalidate the source |
| `borrowing` | `&T`, `&mut T`, borrow rules | many readers XOR one writer, checked at compile time |
| `slices` | `&[T]`, `&str`, ranges | borrow part of a collection without copying |
| `functions_control_flow` | expressions, `if`/`loop`/`for`, labels | nearly everything produces a value |
| `structs` | methods, constructors, newtypes | data and behavior live in separate blocks |
| `enums` | sum types, `Option` | make illegal states unrepresentable |
| `pattern_matching` | `match`, guards, `if let`, `let else` | exhaustiveness is a refactoring tool |
| `error_handling` | `Result`, `?`, custom error enums | errors are values; `panic!` is for bugs |
| `collections` | Vec, HashMap, HashSet, VecDeque | pick by access pattern; `entry` API |
| `strings` | `String` vs `&str`, UTF-8 | bytes are not chars; no integer indexing |
| `iterators` | adapters, consumers, custom iterators | lazy pipelines at zero cost |

## How to use this crate

```sh
# run every test in the crate (unit + integration + doc tests)
cargo test -p fundamentals

# run the tests for one topic while you study it
cargo test -p fundamentals ownership
```

The `compile_fail` blocks in doc comments are real, verified examples
of code the compiler rejects — `cargo test` confirms each one still
fails to compile. They document the *mistakes*, not just the happy
path.

Each module ends with exercises. Do them in a scratch crate
(`cargo new scratch`) or directly in the module — the tests will tell
you if you broke anything.

`tests/learning_checks.rs` shows the difference between unit tests and
integration tests: it can only touch the crate's public API.
