# rust-fundamentals-playground

A Rust learning workspace built around real, compiling, tested code
instead of toy snippets. Every concept lives in a module you can read
top to bottom, run, break, and fix. Common mistakes are part of the
material: `compile_fail` doctests show code the compiler rejects (and
`cargo test` verifies it still rejects them), and every module ends
with exercises.

## Layout

```text
fundamentals/   beginner topics, one module each (in progress)
```

Planned: an `intermediate` crate (traits, generics, lifetimes...),
threading and async crates, and a handful of small projects that put
the pieces together.

## Running things

```sh
cargo test                             # everything
cargo test -p fundamentals ownership   # one topic
```

## Conventions

- Comments explain **why**, not what. If a line needs a "what"
  comment, the line gets rewritten instead.
- Every module ends with exercises; do them in a scratch crate or
  directly in the module — the tests will tell you if you broke
  anything.
