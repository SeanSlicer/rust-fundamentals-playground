//! Ownership: the core idea that makes Rust memory-safe without a
//! garbage collector.
//!
//! Every value has exactly one owner. When the owner goes out of
//! scope, the value is dropped. Assigning or passing a non-`Copy`
//! value *moves* it — the old binding can no longer be used.

/// Passing a `String` by value moves it into the function. The caller
/// gives up ownership; the string is dropped when this function ends
/// (unless we return it, handing ownership back).
pub fn take_ownership(s: String) -> usize {
    s.len()
    // `s` is dropped here. The caller's binding is now invalid.
}

/// The classic move mistake. This is the error every newcomer hits in
/// week one — the fix is usually to borrow (see the `borrowing`
/// module) rather than to clone.
///
/// ```compile_fail
/// let s = String::from("hello");
/// let t = s;            // ownership moves from `s` to `t`
/// println!("{s}");      // error[E0382]: borrow of moved value: `s`
/// ```
// Clippy would inline `t` away — but the named binding is the move
// being demonstrated.
#[allow(clippy::let_and_return)]
pub fn move_semantics() -> String {
    let s = String::from("hello");
    // Moving is just a shallow copy of the (pointer, len, capacity)
    // triple plus invalidating the source — it is cheap, no heap data
    // is copied.
    let t = s;
    t
}

/// `clone` performs a deep copy. It is the explicit escape hatch when
/// you genuinely need two independent values. Rust makes you write
/// `.clone()` so expensive copies are visible in the code.
pub fn clone_when_you_need_two() -> (String, String) {
    let s = String::from("hello");
    let t = s.clone(); // heap data is duplicated here
    (s, t) // both are valid because they own separate allocations
}

/// Types whose values live entirely on the stack (integers, floats,
/// bool, char, and tuples/arrays of them) implement `Copy`: assignment
/// duplicates the bits and the source stays valid. There is no "move"
/// to worry about because copying is as cheap as moving.
pub fn copy_types_do_not_move() -> (i32, i32) {
    let x = 5;
    let y = x; // x is *copied*, not moved
    (x, y) // both usable — this would not compile with String
}

/// Returning a value transfers ownership out of the function. This is
/// how constructors work: the function builds a value and hands it to
/// the caller.
pub fn give_ownership() -> Vec<i32> {
    let v = vec![1, 2, 3];
    v // moved out to the caller; nothing is dropped here
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taking_ownership_consumes_the_value() {
        let s = String::from("hello");
        let len = take_ownership(s);
        // `s` can no longer be used here — uncomment to see E0382:
        // println!("{s}");
        assert_eq!(len, 5);
    }

    #[test]
    fn moves_transfer_not_copy() {
        assert_eq!(move_semantics(), "hello");
    }

    #[test]
    fn clone_yields_independent_values() {
        let (a, b) = clone_when_you_need_two();
        assert_eq!(a, b);
        // They are equal in content but separate allocations: mutating
        // one would never affect the other.
    }

    #[test]
    fn copy_types_stay_usable() {
        assert_eq!(copy_types_do_not_move(), (5, 5));
    }

    #[test]
    fn ownership_can_be_returned() {
        let v = give_ownership();
        assert_eq!(v, vec![1, 2, 3]);
    }
}

// Exercises
// ---------
// 1. Write a function that takes a Vec<String>, appends an element,
//    and returns it. Call it twice in a row on the same vector — note
//    how ownership "threads" through the calls.
// 2. Predict which of these compile, then check: assigning an i32 to
//    two bindings; assigning a String to two bindings; assigning a
//    (i32, String) tuple to two bindings.
// 3. Find a place where you wrote `.clone()` and see if a borrow
//    would work instead (after reading the borrowing module).
