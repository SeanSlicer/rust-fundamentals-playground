//! Integration tests: these live in `tests/` and exercise the crate
//! exactly the way an external user would — only through its public
//! API. Unit tests (inside `src/`, in `#[cfg(test)]` modules) can
//! reach private items; integration tests cannot. Having both kinds
//! is normal: unit tests pin down internals, integration tests pin
//! down the contract.

use fundamentals::{borrowing, collections, error_handling, iterators, slices};

#[test]
fn modules_compose_through_public_api() {
    // Slice helpers feed into iterator pipelines.
    let line = "10 20 thirty 40";
    let first = slices::first_word(line);
    assert_eq!(first, "10");

    let numbers = iterators::parse_valid_numbers(&line.split_whitespace().collect::<Vec<_>>());
    assert_eq!(numbers, vec![10, 20, 40]);
}

#[test]
fn error_types_work_across_crate_boundary() {
    // We can only do this because TemperatureError and its variants
    // are pub — API visibility is part of the design.
    let err = error_handling::parse_celsius("-400").unwrap_err();
    assert!(matches!(
        err,
        error_handling::TemperatureError::BelowAbsoluteZero(_)
    ));
}

#[test]
fn realistic_pipeline_count_then_report() {
    let text = "to be or not to be";
    let top = collections::top_words(text, 2);
    assert_eq!(top[0], ("be".to_string(), 2));
    assert_eq!(top[1], ("to".to_string(), 2));
}

#[test]
fn borrowing_helpers_keep_caller_ownership() {
    let sentence = String::from("integration tests use the public api");
    let len = borrowing::calculate_length_idiomatic(&sentence);
    assert_eq!(len, sentence.len()); // sentence still ours to use
}
