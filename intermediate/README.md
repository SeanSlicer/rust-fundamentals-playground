# intermediate

The abstraction toolkit. Read after `fundamentals`.

| Module | Topic | Key takeaway |
|---|---|---|
| `traits` | shared behavior, default methods, `dyn` | static vs dynamic dispatch is a deliberate choice |
| `generics` | type parameters, bounds, monomorphization | write once, compile per-type, zero runtime cost |
| `lifetimes` | annotations, view structs, `'static` | lifetimes describe; they never extend |
| `closures` | Fn / FnMut / FnOnce, `move`, boxed closures | ask for the weakest trait you can accept |
| `smart_pointers` | Box, Rc, Weak | pick the pointer by its capability |
| `interior_mutability` | Cell, RefCell | borrow rules at runtime; keep borrows short |
| `modules_demo` | visibility, `pub(crate)`, re-exports | privacy is the feature; file layout is incidental |

Suggested order: `traits` → `generics` → `lifetimes`, then the rest as
needed. The two thread-safe siblings of this material — `Arc` and
`Mutex` — live in the `concurrency` crate, next door.

```sh
cargo test -p intermediate            # everything
cargo test -p intermediate lifetimes  # one topic
```

Note the `should_panic` test in `interior_mutability` and the
`compile_fail` doctest in `lifetimes`: failures are part of the
curriculum, and both kinds are verified by `cargo test`.
