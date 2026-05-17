//! Borrowing and references: using a value without taking ownership.
//!
//! A reference is a pointer with compile-time rules attached. The
//! borrow checker enforces, at any given time, EITHER any number of
//! shared references (`&T`) OR exactly one mutable reference
//! (`&mut T`) — never both. This single rule eliminates whole classes
//! of bugs: iterator invalidation, data races, use-after-free.

/// Borrowing instead of moving. Compare with
/// `ownership::take_ownership` — here the caller keeps the string.
// Clippy rightly flags &String (see the idiomatic version below) —
// it stays here so you can compare the two signatures side by side.
#[allow(clippy::ptr_arg)]
pub fn calculate_length(s: &String) -> usize {
    s.len()
    // `s` is a reference; nothing is dropped here.
}

/// Idiomatic version: take `&str`, not `&String`. A `&String` coerces
/// to `&str` automatically, so `&str` accepts string literals, owned
/// strings, and slices alike — strictly more useful, same cost.
pub fn calculate_length_idiomatic(s: &str) -> usize {
    s.len()
}

/// Mutable borrows allow modification through the reference. The
/// caller must own the value mutably (`let mut ...`) and lend it with
/// `&mut`.
pub fn append_exclamation(s: &mut String) {
    s.push('!');
}

/// Shared and mutable borrows cannot overlap. This is the error
/// message you will see most in your first month of Rust:
///
/// ```compile_fail
/// let mut s = String::from("hello");
/// let r1 = &s;          // shared borrow starts
/// let r2 = &mut s;      // error[E0502]: cannot borrow `s` as mutable
///                       // because it is also borrowed as immutable
/// println!("{r1}");     // shared borrow still alive here
/// ```
///
/// The fix is almost always to *shorten* the first borrow — use it and
/// let it end before the mutable borrow starts. Borrows last from
/// creation until their last use (this is called non-lexical
/// lifetimes), so reordering code is often enough.
pub fn borrows_can_be_sequential() -> String {
    let mut s = String::from("hello");

    let len = {
        let r1 = &s; // shared borrow...
        r1.len()
    }; // ...ends here (actually at its last use, even without the block)

    let r2 = &mut s; // now a mutable borrow is fine
    r2.push_str(&format!(" ({len} chars)"));
    s
}

/// References must never outlive the data they point to. The compiler
/// rejects dangling references at compile time:
///
/// ```compile_fail
/// fn dangle() -> &String {
///     let s = String::from("hello");
///     &s // error[E0106]: `s` is dropped at the end of the function
/// }
/// ```
///
/// The fix is to return the owned value instead — hand ownership to
/// the caller rather than a pointer into a dead stack frame.
// The named binding mirrors the compile_fail example above —
// clippy's "inline it" suggestion would destroy the parallel.
#[allow(clippy::let_and_return)]
pub fn no_dangling_fix() -> String {
    let s = String::from("hello");
    s // move ownership out: the value survives, no reference needed
}

