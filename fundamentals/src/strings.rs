//! Strings: `String` vs `&str`, UTF-8, and why `s[0]` doesn't exist.
//!
//! `String` is an owned, growable, heap-allocated buffer. `&str` is a
//! borrowed view into string data (a slice). Both are guaranteed
//! valid UTF-8 — which is exactly why Rust refuses to let you index a
//! string by integer: byte 0 of "héllo" is a full char, but byte 1 is
//! the middle of one.

/// The conversions you use every day. Take `&str` in parameters,
/// return `String` when you allocate — that's the default calling
/// convention for string-handling code.
pub fn shout(input: &str) -> String {
    input.to_uppercase()
}

/// Building strings: `push_str` for slices, `push` for single chars,
/// `format!` when combining several pieces. `+` works but moves the
/// left operand and reads poorly past two pieces.
pub fn full_name(first: &str, last: &str) -> String {
    // format! never takes ownership of its arguments — borrow-friendly.
    format!("{last}, {first}")
}

/// Why no indexing: strings are UTF-8 bytes, and chars vary from 1 to
/// 4 bytes. `len()` is BYTES, not characters — a frequent source of
/// off-by-N bugs with any non-ASCII text.
pub fn byte_and_char_counts(s: &str) -> (usize, usize) {
    (s.len(), s.chars().count())
}

/// To get the "nth character" you must iterate. This is O(n) — Rust
/// makes the cost visible instead of hiding it behind `s[n]`.
pub fn nth_char(s: &str, n: usize) -> Option<char> {
    s.chars().nth(n)
}

/// Slicing by byte ranges works, but panics if a boundary lands inside
/// a multi-byte char. Only slice at boundaries you know are safe
/// (e.g. indices from `find`, which always returns boundary offsets).
pub fn domain_of(email: &str) -> Option<&str> {
    let at = email.find('@')?;
    // at + 1 is safe: '@' is 1 byte, so this is a char boundary.
    Some(&email[at + 1..])
}

/// Splitting and trimming — the bread and butter of input parsing.
/// Note `split_whitespace` handles runs of spaces and leading/trailing
/// space, which `split(' ')` does not (it yields empty strings).
pub fn parse_tags(line: &str) -> Vec<String> {
    line.split(',')
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect()
}

/// Comparing the iteration views: chars() yields Unicode scalar
/// values, bytes() yields raw u8s. Pick based on what you are
/// actually processing.
pub fn is_ascii_digit_string(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owned_and_borrowed_conversions() {
        assert_eq!(shout("hello"), "HELLO");
        assert_eq!(full_name("Ada", "Lovelace"), "Lovelace, Ada");
    }

    #[test]
    fn bytes_are_not_chars() {
        // 5 characters, but é is 2 bytes in UTF-8.
        assert_eq!(byte_and_char_counts("héllo"), (6, 5));
        // Plain ASCII: the counts agree, which is why this bug hides.
        assert_eq!(byte_and_char_counts("hello"), (5, 5));
    }

    #[test]
    fn nth_char_iterates() {
        assert_eq!(nth_char("héllo", 1), Some('é'));
        assert_eq!(nth_char("hi", 5), None);
    }

    #[test]
    fn slicing_at_known_boundaries() {
        assert_eq!(domain_of("user@example.com"), Some("example.com"));
        assert_eq!(domain_of("not-an-email"), None);
    }

    #[test]
    fn parsing_splits_and_trims() {
        assert_eq!(
            parse_tags(" rust , beginner ,, notes "),
            ["rust", "beginner", "notes"]
        );
        assert!(parse_tags("  ").is_empty());
    }

    #[test]
    fn byte_level_checks() {
        assert!(is_ascii_digit_string("12345"));
        assert!(!is_ascii_digit_string("12a45"));
        assert!(!is_ascii_digit_string(""));
    }
}

// Exercises
// ---------
// 1. Write `fn reverse(s: &str) -> String`. Test it with "héllo" —
//    does chars().rev() do what you expect?
// 2. Write a function that truncates a string to at most n CHARACTERS
//    (not bytes) without panicking. `char_indices` is your friend.
// 3. Why does `let c = "hé"[1];` not compile while `let b =
//    "hé".as_bytes()[1];` works? Write the answer as a comment.
