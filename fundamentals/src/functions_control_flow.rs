//! Functions and control flow.
//!
//! The key insight: almost everything in Rust is an *expression* — it
//! produces a value. `if`, `match`, blocks, and loops can all appear
//! on the right side of `let`. Once this clicks, a lot of Rust style
//! makes sense.

/// The last expression of a function body (no semicolon!) is the
/// return value. Adding a semicolon turns it into a statement and the
/// function returns `()` — a very common beginner compile error.
pub fn add(a: i32, b: i32) -> i32 {
    a + b // no `return`, no semicolon: this IS the return value
}

/// `if` is an expression, so there is no ternary operator — you don't
/// need one. Both branches must have the same type.
pub fn absolute(n: i32) -> i32 {
    if n < 0 {
        -n
    } else {
        n
    }
}

/// Blocks are expressions too. This is idiomatic for computing a value
/// that needs a few intermediate steps without leaking temporaries
/// into the surrounding scope.
pub fn hypotenuse(a: f64, b: f64) -> f64 {
    let sum_of_squares = {
        let a2 = a * a;
        let b2 = b * b;
        a2 + b2 // block evaluates to this
    };
    sum_of_squares.sqrt()
}

/// `loop` is an expression that can yield a value through `break`.
/// Use `loop` (not `while true`) when the exit condition lives in the
/// middle of the body — the compiler also understands `loop` better
/// for definite-assignment analysis.
pub fn first_power_of_two_above(threshold: u32) -> u32 {
    let mut value = 1u32;
    loop {
        if value > threshold {
            break value; // `loop` evaluates to this
        }
        value *= 2;
    }
}

/// `for` iterates anything that implements IntoIterator. Prefer it
/// over index-based `while` loops: no off-by-one errors, no bounds
/// checks in the hot path, and it works with non-indexable iterators.
pub fn count_vowels(text: &str) -> usize {
    let mut count = 0;
    for c in text.chars() {
        if matches!(c, 'a' | 'e' | 'i' | 'o' | 'u') {
            count += 1;
        }
    }
    count
}

/// Loop labels disambiguate `break`/`continue` in nested loops.
/// Without the label, `break` would only exit the inner loop.
pub fn find_pair_summing_to(numbers: &[i32], target: i32) -> Option<(i32, i32)> {
    let mut found = None;
    'outer: for (i, &a) in numbers.iter().enumerate() {
        for &b in &numbers[i + 1..] {
            if a + b == target {
                found = Some((a, b));
                break 'outer; // exits BOTH loops
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expressions_return_values() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(absolute(-7), 7);
        assert_eq!(absolute(7), 7);
    }

    #[test]
    fn block_expression_computes_hypotenuse() {
        assert!((hypotenuse(3.0, 4.0) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn loop_yields_a_value_via_break() {
        assert_eq!(first_power_of_two_above(100), 128);
        assert_eq!(first_power_of_two_above(0), 1);
    }

    #[test]
    fn for_loops_iterate_chars() {
        assert_eq!(count_vowels("rustacean"), 4);
        assert_eq!(count_vowels("xyz"), 0);
    }

    #[test]
    fn labeled_break_exits_nested_loops() {
        assert_eq!(find_pair_summing_to(&[1, 2, 3, 4], 7), Some((3, 4)));
        assert_eq!(find_pair_summing_to(&[1, 2], 100), None);
    }
}

// Exercises
// ---------
// 1. Rewrite `count_vowels` with iterator adapters (filter + count) —
//    see the iterators module for the vocabulary.
// 2. Write fizzbuzz returning a Vec<String>, using `match (i % 3,
//    i % 5)` instead of an if/else chain.
// 3. Write a function that finds the first duplicate in a slice using
//    a labeled loop, then again using a HashSet. Which reads better?
