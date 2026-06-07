# calc

A command-line calculator built on a hand-written tokenizer and
recursive descent parser.

```sh
cargo run -p calc -- "(1 + 2) * -3.5"   # one-shot
cargo run -p calc                        # REPL
cargo test -p calc
```

## What it teaches

- **Enums as vocabularies** — `Token` and `CalcError` each enumerate
  everything that can occur, and `match` proves every case is handled.
- **Recursive descent** — the grammar's precedence levels become the
  call hierarchy `expr → term → factor`. Read the grammar comment in
  `lib.rs` first; the code follows it line for line.
- **Errors as values** — seven distinct failure modes, each tested.
  The binary maps them to stderr + a non-zero exit code.
- **lib/bin split** — logic in `lib.rs` (testable), I/O in `main.rs`
  (thin). This is the layout to copy for your own CLI tools.

## Suggested experiments

Start with the exercises at the bottom of `lib.rs` (add `%`, add a
right-associative `^`, split out an AST). The AST refactor is the big
one — it turns this from a calculator into the front half of an
interpreter.
