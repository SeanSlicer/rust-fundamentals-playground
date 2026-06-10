# log-parser

Parse `date time [LEVEL] message` logs and print a summary report.

```sh
cargo run -p log-parser -- sample.log
cargo test -p log-parser
```

## What it teaches

- **`FromStr`** — implement the standard parsing trait and
  `"ERROR".parse::<Level>()` just works. The Display impl makes the
  round trip symmetric, which the tests pin down.
- **Best-effort parsing** — `parse_log` returns
  `(Vec<Entry>, Vec<BadLine>)` instead of `Result`. Real logs contain
  garbage; a tool that aborts on line 2 of a million-line file is
  useless on exactly the files that matter. Compare with the CSV
  parser next door, which is strict — choosing strict vs lenient *is*
  the design decision.
- **`split_once` chains** — peeling fields off the front of a line
  beats index arithmetic; `sample.log` includes a message containing
  `[brackets]` to show why "split at the FIRST `] `" matters.
- **Deterministic output from a HashMap** — the binary iterates levels
  in severity order because HashMap order is unstable by design.

`sample.log` includes a timestamp-less panic line on purpose — run the
binary and find it in the "unparseable lines" section.
