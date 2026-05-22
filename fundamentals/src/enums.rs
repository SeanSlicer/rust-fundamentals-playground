//! Enums: sum types — a value that is exactly one of several variants.
//!
//! Rust enums are far more powerful than C-style enums: each variant
//! can carry its own data. Combined with `match`, they make illegal
//! states unrepresentable. If you find yourself writing a struct full
//! of Options where only certain combinations are valid, you probably
//! want an enum.

/// Variants can be unit-like, tuple-like, or struct-like — all in the
/// same enum.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// Unit variant: no data.
    Point,
    /// Tuple variant: positional data.
    Circle(f64),
    /// Struct variant: named fields, good when meaning isn't obvious
    /// from position.
    Rectangle { width: f64, height: f64 },
}

impl Shape {
    /// Methods on enums almost always start with a `match`. The match
    /// must be exhaustive — adding a new variant later causes a
    /// compile error in every match that doesn't handle it. That is a
    /// feature: the compiler walks you to every place that needs
    /// updating.
    pub fn area(&self) -> f64 {
        match self {
            Shape::Point => 0.0,
            Shape::Circle(radius) => std::f64::consts::PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
        }
    }
}

/// Modeling states as an enum instead of booleans. Compare with
/// `struct Connection { connected: bool, connecting: bool, addr:
/// Option<String> }` — that struct has invalid combinations (both
/// flags true, connected without an addr); this enum has none.
#[derive(Debug, Clone, PartialEq)]
pub enum Connection {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { peer: String },
}

impl Connection {
    /// State transitions become total functions over the enum. Every
    /// (state, event) combination is visibly handled.
    pub fn retry(self) -> Connection {
        match self {
            Connection::Disconnected => Connection::Connecting { attempt: 1 },
            Connection::Connecting { attempt } => Connection::Connecting {
                attempt: attempt + 1,
            },
            // Retrying while connected is a no-op; returning self keeps
            // the peer string without cloning.
            already_connected @ Connection::Connected { .. } => already_connected,
        }
    }
}

/// `Option<T>` is just an enum from the standard library:
/// `enum Option<T> { None, Some(T) }`. Rust has no null — a value that
/// might be absent must say so in its type, and the compiler forces
/// you to handle the None case before touching the T.
// Clippy knows this is `.find()` in disguise — exercise 3 asks you to
// make that rewrite yourself, so the manual loop stays.
#[allow(clippy::manual_find)]
pub fn first_even(numbers: &[i32]) -> Option<i32> {
    for &n in numbers {
        if n % 2 == 0 {
            return Some(n);
        }
    }
    None
}

/// Option has a rich combinator API; reach for these before writing a
/// `match` on a single Option. `map` transforms the inside, `unwrap_or`
/// supplies a fallback.
pub fn first_even_doubled_or_zero(numbers: &[i32]) -> i32 {
    first_even(numbers).map(|n| n * 2).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_carry_different_data() {
        assert_eq!(Shape::Point.area(), 0.0);
        assert!((Shape::Circle(1.0).area() - std::f64::consts::PI).abs() < 1e-10);
        assert_eq!(
            Shape::Rectangle {
                width: 2.0,
                height: 3.0
            }
            .area(),
            6.0
        );
    }

    #[test]
    fn state_machine_transitions() {
        let c = Connection::Disconnected.retry();
        assert_eq!(c, Connection::Connecting { attempt: 1 });
        let c = c.retry();
        assert_eq!(c, Connection::Connecting { attempt: 2 });

        let connected = Connection::Connected {
            peer: "10.0.0.1".into(),
        };
        assert_eq!(connected.clone().retry(), connected);
    }

    #[test]
    fn option_makes_absence_explicit() {
        assert_eq!(first_even(&[1, 3, 4, 5]), Some(4));
        assert_eq!(first_even(&[1, 3, 5]), None);
    }

    #[test]
    fn option_combinators_avoid_matches() {
        assert_eq!(first_even_doubled_or_zero(&[1, 4]), 8);
        assert_eq!(first_even_doubled_or_zero(&[1, 3]), 0);
    }
}

// Exercises
// ---------
// 1. Add a `Triangle { base: f64, height: f64 }` variant to Shape.
//    Follow the compile errors — they point at every match that needs
//    a new arm.
// 2. Model a traffic light as an enum with a `next()` method and a
//    `duration_secs()` method.
// 3. Rewrite `first_even` using iterator adapters:
//    `numbers.iter().copied().find(|n| n % 2 == 0)`.
