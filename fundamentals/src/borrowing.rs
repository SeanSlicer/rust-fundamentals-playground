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

// ---------------------------------------------------------------------------
// The borrowing rules, restated
// ---------------------------------------------------------------------------
//
// 1. At any time: many `&T` XOR one `&mut T`.
// 2. References must always be valid (no dangling).
//
// A useful mental model: `&T` is a reader lock, `&mut T` is a writer
// lock, and the borrow checker is a lock manager that runs entirely at
// compile time. When fighting the borrow checker, ask "who is reading
// while I am trying to write?" — the answer is usually a reference you
// created earlier and are still holding.

/// Why the rule matters in practice: pushing to a Vec may reallocate
/// and move every element, which would leave any outstanding reference
/// pointing at freed memory. The borrow checker stops this at compile
/// time:
///
/// ```compile_fail
/// let mut v = vec![1, 2, 3];
/// let first = &v[0];   // shared borrow into the buffer
/// v.push(4);           // error[E0502]: push needs &mut v
/// println!("{first}"); // borrow still alive
/// ```
pub fn safe_alternative_to_hold_then_push() -> i32 {
    let mut v = vec![1, 2, 3];
    // Copy the element out (i32 is Copy) instead of holding a
    // reference across the mutation. The borrow ends immediately.
    let first = v[0];
    v.push(4);
    first + v.len() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowing_leaves_caller_with_ownership() {
        let s = String::from("hello");
        let len = calculate_length(&s);
        // `s` is still usable — we only lent it.
        assert_eq!(len, 5);
        assert_eq!(s, "hello");
    }

    #[test]
    fn str_slice_accepts_everything() {
        let owned = String::from("hello");
        assert_eq!(calculate_length_idiomatic(&owned), 5);
        assert_eq!(calculate_length_idiomatic("literal"), 7);
    }

    #[test]
    fn mutable_borrow_modifies_in_place() {
        let mut s = String::from("hi");
        append_exclamation(&mut s);
        assert_eq!(s, "hi!");
    }

    #[test]
    fn sequential_borrows_compile() {
        assert_eq!(borrows_can_be_sequential(), "hello (5 chars)");
    }

    #[test]
    fn copy_out_instead_of_holding_a_borrow() {
        assert_eq!(safe_alternative_to_hold_then_push(), 5);
    }
}

// Exercises
// ---------
// 1. Write `fn first_word(s: &str) -> &str` that returns everything up
//    to the first space. (Peek at the slices module if stuck.)
// 2. Take the compile_fail example with Vec::push and fix it three
//    different ways: copy the value out, shorten the borrow, or push
//    before borrowing.
// 3. Write a function taking `&mut Vec<i32>` that removes all negative
//    numbers in place (hint: `retain`).
