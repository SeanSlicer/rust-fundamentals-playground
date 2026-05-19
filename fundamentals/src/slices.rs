//! Slices: references to a contiguous chunk of a collection.
//!
//! A slice (`&[T]` or `&str`) is a fat pointer: an address plus a
//! length. It borrows part (or all) of a collection without copying.
//! Slices are the reason Rust APIs rarely take `&Vec<T>` or `&String`
//! — the slice type is strictly more general.

/// The canonical slice example from the book: return the first word of
/// a string as a slice *into* the original. No allocation happens; the
/// return value borrows from `s`, so `s` cannot be mutated while the
/// returned slice is alive.
pub fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(index) => &s[..index],
        None => s,
    }
}

/// Array and Vec slices work the same way as string slices. Taking
/// `&[i32]` lets this function accept arrays, vectors, and other
/// slices without caring which one the caller has.
pub fn sum_of(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

/// Ranges select sub-slices. `..n` is "from the start", `n..` is "to
/// the end", `..` is the whole thing. Out-of-bounds slicing panics at
/// runtime, so prefer `get` when the range comes from user input.
pub fn middle(numbers: &[i32]) -> &[i32] {
    if numbers.len() < 3 {
        return numbers;
    }
    &numbers[1..numbers.len() - 1]
}

/// `get` returns Option instead of panicking — the safe counterpart to
/// indexing. Use indexing when out-of-bounds is a bug; use `get` when
/// it is an expected case to handle.
pub fn safe_lookup(numbers: &[i32], index: usize) -> Option<i32> {
    numbers.get(index).copied()
}

/// Mutable slices allow in-place modification of the borrowed region.
pub fn double_all(numbers: &mut [i32]) {
    for n in numbers.iter_mut() {
        *n *= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_word_borrows_from_input() {
        assert_eq!(first_word("hello world"), "hello");
        assert_eq!(first_word("single"), "single");
        assert_eq!(first_word(""), "");
    }

    #[test]
    fn slices_accept_arrays_and_vecs() {
        let array = [1, 2, 3];
        let vec = vec![4, 5, 6];
        // &array and &vec both coerce to &[i32].
        assert_eq!(sum_of(&array), 6);
        assert_eq!(sum_of(&vec), 15);
        // And sub-slices work too:
        assert_eq!(sum_of(&vec[1..]), 11);
    }

    #[test]
    fn middle_drops_first_and_last() {
        assert_eq!(middle(&[1, 2, 3, 4]), &[2, 3]);
        assert_eq!(middle(&[1, 2]), &[1, 2]);
    }

    #[test]
    fn get_is_the_non_panicking_index() {
        let v = [10, 20];
        assert_eq!(safe_lookup(&v, 1), Some(20));
        assert_eq!(safe_lookup(&v, 9), None);
    }

    #[test]
    fn mutable_slices_modify_in_place() {
        let mut v = vec![1, 2, 3];
        double_all(&mut v);
        assert_eq!(v, vec![2, 4, 6]);
    }
}

// Exercises
// ---------
// 1. Write `fn last_word(s: &str) -> &str` (hint: `rfind`).
// 2. Write `fn split_at_middle(s: &[i32]) -> (&[i32], &[i32])` and
//    compare with the built-in `split_at`.
// 3. Why does `fn longest_prefix(a: &str, b: &str) -> &str` need a
//    lifetime annotation while `first_word` does not? Come back to
//    this one after the lifetimes module.
