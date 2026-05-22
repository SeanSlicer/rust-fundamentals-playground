//! Pattern matching: destructuring values and branching on shape.
//!
//! Patterns appear in `match`, `if let`, `while let`, `let`, function
//! parameters, and `for` loops. `match` is the workhorse: exhaustive,
//! expression-based, and checked by the compiler.

/// Matching on values and ranges. `_` is the catch-all; the compiler
/// requires the match to cover every possible u8.
pub fn describe_byte(b: u8) -> &'static str {
    match b {
        0 => "zero",
        b'a'..=b'z' => "lowercase ascii letter",
        b'A'..=b'Z' => "uppercase ascii letter",
        b'0'..=b'9' => "ascii digit",
        128.. => "not ascii",
        _ => "other ascii",
    }
}

/// Destructuring tuples in a match. Matching on a tuple of values is
/// the idiomatic way to branch on *combinations* — far clearer than
/// nested if/else.
pub fn fizzbuzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => "fizzbuzz".to_string(),
        (0, _) => "fizz".to_string(),
        (_, 0) => "buzz".to_string(),
        _ => n.to_string(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Click { x: i32, y: i32 },
    KeyPress(char),
    Paste(String),
    Resize { width: u32, height: u32 },
}

/// One function showing most of the pattern toolbox:
/// struct destructuring, guards, `@` bindings, and or-patterns.
pub fn handle(event: &Event) -> String {
    match event {
        // Guard: extra boolean condition on top of the pattern.
        Event::Click { x, y } if *x < 0 || *y < 0 => "click outside window".to_string(),
        Event::Click { x, y } => format!("click at ({x}, {y})"),

        // Or-patterns: several literals sharing one arm.
        Event::KeyPress('q' | 'Q') => "quit".to_string(),

        // `@` binds the matched value to a name while also testing it.
        Event::KeyPress(c @ '0'..='9') => format!("digit {c}"),
        Event::KeyPress(c) => format!("key {c}"),

        // Match on a property of the data, not just its shape.
        Event::Paste(s) if s.len() > 10 => "large paste".to_string(),
        Event::Paste(s) => format!("paste: {s}"),

        // `..` ignores remaining fields explicitly.
        Event::Resize { width, .. } => format!("resize to width {width}"),
    }
}

/// `if let` is a one-arm match: use it when you only care about a
/// single pattern and exhaustiveness would be noise. `else` handles
/// the rest.
pub fn double_if_some(value: Option<i32>) -> i32 {
    if let Some(n) = value {
        n * 2
    } else {
        0
    }
}

/// `let else` binds a pattern or diverges — the idiomatic early-return
/// guard. The happy path stays unindented.
// Clippy notes this particular let-else could be `?` — true, but the
// point is the let-else shape itself, which also works in functions
// that do not return Option.
#[allow(clippy::question_mark)]
pub fn parse_pair(input: &str) -> Option<(i32, i32)> {
    let Some((left, right)) = input.split_once(',') else {
        return None;
    };
    let left = left.trim().parse().ok()?;
    let right = right.trim().parse().ok()?;
    Some((left, right))
}

/// `while let` loops as long as the pattern matches — the standard way
/// to drain a stack or read from an iterator manually.
pub fn drain_stack(mut stack: Vec<i32>) -> Vec<i32> {
    let mut popped = Vec::new();
    while let Some(top) = stack.pop() {
        popped.push(top);
    }
    popped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_and_catch_alls() {
        assert_eq!(describe_byte(0), "zero");
        assert_eq!(describe_byte(b'm'), "lowercase ascii letter");
        assert_eq!(describe_byte(b'7'), "ascii digit");
        assert_eq!(describe_byte(200), "not ascii");
    }

    #[test]
    fn tuple_matching_for_combinations() {
        assert_eq!(fizzbuzz(15), "fizzbuzz");
        assert_eq!(fizzbuzz(9), "fizz");
        assert_eq!(fizzbuzz(10), "buzz");
        assert_eq!(fizzbuzz(7), "7");
    }

    #[test]
    fn guards_bindings_and_or_patterns() {
        assert_eq!(handle(&Event::Click { x: 3, y: 4 }), "click at (3, 4)");
        assert_eq!(
            handle(&Event::Click { x: -1, y: 4 }),
            "click outside window"
        );
        assert_eq!(handle(&Event::KeyPress('Q')), "quit");
        assert_eq!(handle(&Event::KeyPress('5')), "digit 5");
        assert_eq!(handle(&Event::Paste("hi".into())), "paste: hi");
        assert_eq!(
            handle(&Event::Resize {
                width: 80,
                height: 24
            }),
            "resize to width 80"
        );
    }

    #[test]
    fn if_let_and_let_else() {
        assert_eq!(double_if_some(Some(21)), 42);
        assert_eq!(double_if_some(None), 0);
        assert_eq!(parse_pair("3, 4"), Some((3, 4)));
        assert_eq!(parse_pair("nope"), None);
        assert_eq!(parse_pair("3, x"), None);
    }

    #[test]
    fn while_let_drains_in_lifo_order() {
        assert_eq!(drain_stack(vec![1, 2, 3]), vec![3, 2, 1]);
    }
}

// Exercises
// ---------
// 1. Add a `Scroll { lines: i32 }` event where negative lines mean
//    scrolling up — handle both directions with a guard.
// 2. Write a match over `(Option<i32>, Option<i32>)` that adds the
//    numbers when both are present, returns the present one when only
//    one is, and 0 otherwise.
// 3. Rewrite `parse_pair` without `let else` or `?`, using only
//    `match`. Compare the lengths.
