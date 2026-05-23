//! Error handling with `Option`, `Result`, and the `?` operator.
//!
//! Rust has no exceptions. Fallible operations return
//! `Result<T, E>` and the type system forces callers to acknowledge
//! the failure case. The division of labor:
//!
//! * `Option<T>` — a value might be absent, and absence is not an
//!   error worth explaining (lookup misses, empty input).
//! * `Result<T, E>` — an operation can fail and the caller deserves
//!   to know *why*.
//! * `panic!` — a bug. Unrecoverable, should never happen in correct
//!   code (index out of bounds, broken invariant). Don't use panics
//!   for expected failures like bad user input.

/// Returning Result instead of panicking. The error type here is a
/// simple String — fine for examples and small tools; real libraries
/// define error enums (see below).
pub fn divide(dividend: f64, divisor: f64) -> Result<f64, String> {
    if divisor == 0.0 {
        // The Err variant carries the explanation.
        return Err(String::from("cannot divide by zero"));
    }
    Ok(dividend / divisor)
}

/// The `?` operator: if the Result is Err, return it from the current
/// function immediately; otherwise unwrap the Ok. It removes the
/// match-and-rethrow boilerplate that dominates error-handling code in
/// other languages.
pub fn average(values: &[f64]) -> Result<f64, String> {
    if values.is_empty() {
        return Err(String::from("cannot average an empty slice"));
    }
    let sum: f64 = values.iter().sum();
    // `?` propagates the division error upward; on success we keep
    // going with the unwrapped f64.
    let avg = divide(sum, values.len() as f64)?;
    Ok(avg)
}

/// `?` works on Option too — `None` short-circuits out of the
/// function. Mixing Option and Result in one function requires
/// converting (`ok_or`), shown here.
pub fn first_word_length(text: &str) -> Result<usize, String> {
    let first = text
        .split_whitespace()
        .next()
        // Convert Option -> Result by attaching an error message.
        .ok_or_else(|| String::from("input was empty"))?;
    Ok(first.len())
}

/// When NOT to use unwrap: on anything that can fail for reasons
/// outside your control. When unwrap/expect is fine: tests, examples,
/// and cases where failure is provably impossible — and then prefer
/// `expect` with a message explaining *why* it cannot fail.
pub fn parse_known_good() -> i32 {
    // This literal is guaranteed to parse; expect documents that
    // reasoning for the next reader.
    "42".parse().expect("literal is a valid i32")
}

