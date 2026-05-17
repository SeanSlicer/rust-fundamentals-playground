//! Variables, mutability, shadowing, and constants.
//!
//! Rust variables are immutable by default. That is not a limitation —
//! it is a guarantee: anyone reading the code knows a `let` binding
//! never changes unless it is explicitly marked `mut`.

/// Constants must have an explicit type and are evaluated at compile
/// time. By convention they are SCREAMING_SNAKE_CASE. Use a constant
/// when a value is conceptually fixed for the whole program.
pub const SECONDS_PER_MINUTE: u32 = 60;

/// Demonstrates the difference between an immutable binding and a
/// mutable one.
pub fn mutability() -> i32 {
    // Immutable by default. Trying to reassign `x` below would not
    // compile:
    //
    // ```compile_fail
    // let x = 5;
    // x = 6; // error[E0384]: cannot assign twice to immutable variable
    // ```
    let x = 5;

    // Opt in to mutation with `mut`. Reach for `mut` only when you
    // actually need it — most Rust code transforms values into new
    // bindings instead of mutating in place.
    let mut y = x;
    y += 1;
    y
}

/// Shadowing re-declares a name with a *new* binding. Unlike `mut`,
/// shadowing can change the type, and the previous value becomes
/// unreachable. Idiomatic Rust uses shadowing for "refinement"
/// pipelines: parse a string, then keep working with the number under
/// the same name.
// Clippy would collapse the last shadowing step into the return —
// correct in normal code, but here each step IS the demonstration.
#[allow(clippy::let_and_return)]
pub fn shadowing(input: &str) -> usize {
    // `input` starts life as a &str...
    let input = input.trim();
    // ...and is shadowed again as a usize. A `mut` variable could not
    // do this, because mutation never changes a variable's type.
    let input = input.len();
    input
}

/// Type annotations are usually optional thanks to inference, but you
/// need them when the compiler has no information to infer from —
/// `parse` is the classic example because it can produce many types.
pub fn annotated_parse(text: &str) -> Option<i64> {
    // Without `: i64` (or a turbofish `parse::<i64>()`) this would be
    // ambiguous: parse into what?
    let value: i64 = text.trim().parse().ok()?;
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutability_returns_incremented_value() {
        assert_eq!(mutability(), 6);
    }

    #[test]
    fn shadowing_changes_type_under_same_name() {
        assert_eq!(shadowing("  hello  "), 5);
    }

    #[test]
    fn annotated_parse_handles_good_and_bad_input() {
        assert_eq!(annotated_parse(" 42 "), Some(42));
        assert_eq!(annotated_parse("not a number"), None);
    }

    #[test]
    fn constants_are_just_values() {
        assert_eq!(SECONDS_PER_MINUTE * 60, 3600);
    }
}

// Exercises
// ---------
// 1. Write a function that takes a Celsius temperature as a string,
//    parses it, converts it to Fahrenheit, and returns the result —
//    using shadowing instead of new variable names at each step.
// 2. Try to make `shadowing` work with `mut` instead. Notice where the
//    compiler stops you and why.
// 3. Add a constant for SECONDS_PER_DAY built from SECONDS_PER_MINUTE
//    and verify it in a test.
