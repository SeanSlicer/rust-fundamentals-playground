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

// ---------------------------------------------------------------------------
// Custom error types
// ---------------------------------------------------------------------------
// String errors don't compose: callers can't match on them and they
// lose the underlying cause. The idiomatic upgrade is an error enum
// with one variant per failure mode, implementing Display and Error.

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum TemperatureError {
    /// Wraps the std parse error so the cause isn't thrown away.
    NotANumber(std::num::ParseFloatError),
    /// Domain validation: physically impossible value.
    BelowAbsoluteZero(f64),
}

impl fmt::Display for TemperatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemperatureError::NotANumber(e) => write!(f, "not a number: {e}"),
            TemperatureError::BelowAbsoluteZero(c) => {
                write!(f, "{c}°C is below absolute zero (-273.15°C)")
            }
        }
    }
}

// Implementing std::error::Error makes the type interoperable with
// the wider ecosystem (Box<dyn Error>, anyhow, error reporters).
impl std::error::Error for TemperatureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TemperatureError::NotANumber(e) => Some(e),
            TemperatureError::BelowAbsoluteZero(_) => None,
        }
    }
}

// `From` is what powers `?`-conversion: with this impl, a function
// returning TemperatureError can use `?` directly on a parse result
// and the ParseFloatError is converted automatically.
impl From<std::num::ParseFloatError> for TemperatureError {
    fn from(e: std::num::ParseFloatError) -> Self {
        TemperatureError::NotANumber(e)
    }
}

/// Parse and validate a Celsius temperature. Note how clean the body
/// is: `?` handles the parse failure via the From impl above, leaving
/// only the domain logic visible.
pub fn parse_celsius(input: &str) -> Result<f64, TemperatureError> {
    let celsius: f64 = input.trim().parse()?; // ParseFloatError -> TemperatureError
    if celsius < -273.15 {
        return Err(TemperatureError::BelowAbsoluteZero(celsius));
    }
    Ok(celsius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divide_reports_division_by_zero() {
        assert_eq!(divide(10.0, 4.0), Ok(2.5));
        assert!(divide(1.0, 0.0).is_err());
    }

    #[test]
    fn question_mark_propagates_errors() {
        assert_eq!(average(&[1.0, 2.0, 3.0]), Ok(2.0));
        assert!(average(&[]).is_err());
    }

    #[test]
    fn option_to_result_conversion() {
        assert_eq!(first_word_length("hello world"), Ok(5));
        assert!(first_word_length("   ").is_err());
    }

    #[test]
    fn expect_documents_impossibility() {
        assert_eq!(parse_known_good(), 42);
    }

    #[test]
    fn custom_errors_are_matchable() {
        assert_eq!(parse_celsius(" 21.5 "), Ok(21.5));

        // Callers can distinguish failure modes — impossible with
        // String errors.
        match parse_celsius("-300") {
            Err(TemperatureError::BelowAbsoluteZero(c)) => assert_eq!(c, -300.0),
            other => panic!("expected BelowAbsoluteZero, got {other:?}"),
        }
        assert!(matches!(
            parse_celsius("warm"),
            Err(TemperatureError::NotANumber(_))
        ));
    }

    #[test]
    fn custom_errors_display_nicely() {
        let err = parse_celsius("-300").unwrap_err();
        assert_eq!(err.to_string(), "-300°C is below absolute zero (-273.15°C)");
    }
}

// Exercises
// ---------
// 1. Add a `TooHot(f64)` variant rejecting temperatures above 1000°C.
//    The compiler will point you at every match needing a new arm.
// 2. Write `fn read_number_from_file(path: &str) -> Result<i32, ...>`
//    that needs to combine io::Error and ParseIntError into one enum.
// 3. Replace the String errors in `divide`/`average` with a proper
//    enum. Was anything lost? Anything gained?
