# csv-parser

A hand-written CSV parser with RFC 4180-style quoting, built to learn
what the real `csv` crate does under the hood.

```sh
cargo run -p csv-parser -- people.csv
cargo test -p csv-parser
```

## What it teaches

- **State machines with enums** — the parser is a single `match` over
  `(State, char)`. The `State` enum makes impossible states
  unrepresentable, and exhaustiveness checking proves every
  transition is handled. This is the most transferable pattern in the
  whole workspace: lexers, protocol decoders, and input handlers all
  look exactly like this.
- **Position-aware errors** — every error carries a line (and column
  where it has one), because "bad file" is useless and "line 7,
  column 3" is actionable.
- **`std::mem::take`** — move a String out of a mut variable and
  leave an empty one behind: the idiomatic no-clone way to "flush" an
  accumulator.

## Why quoting is the whole problem

`line.split(',')` handles the happy case in one line. Then someone's
note field contains a comma. The included `people.csv` has commas
*and* escaped quotes inside fields — run the binary on it and trace
the state machine by hand for row 2. That exercise is the crate.
