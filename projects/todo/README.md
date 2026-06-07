# todo-cli

A todo list that persists to `todos.json` in the working directory.

```sh
cargo run -p todo-cli -- add buy milk
cargo run -p todo-cli -- add "learn rust properly"
cargo run -p todo-cli            # list (the default)
cargo run -p todo-cli -- done 1
cargo run -p todo-cli -- remove 2
cargo test -p todo-cli
```

## What it teaches

- **Serde round-tripping** — derive `Serialize`/`Deserialize` and the
  structs *are* the file format. `#[serde(default)]` shows how to
  evolve that format without breaking old files.
- **Option-based domain errors** — `mark_done`/`remove` return
  `Option`; "no such id" is an expected case, not a panic.
- **Edge vs core** — `TodoList` never touches a file; `load`/`save`
  never touch list logic. Tests for each are simple because of it.
- **A useful io::Error pattern** — `load` treats `NotFound` as "fresh
  list" instead of an error, by matching on `e.kind()`.
- **Slice patterns for CLI parsing** — see `parse_args` in `main.rs`;
  no argument-parsing crate needed at this size.

The stable-id design (ids never reused, survive removals) is worth a
look — it is the same reasoning behind database primary keys, in
thirty lines.
