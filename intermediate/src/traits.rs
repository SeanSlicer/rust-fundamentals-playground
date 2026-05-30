//! Traits: shared behavior without inheritance.
//!
//! A trait is a contract: "any type implementing me provides these
//! methods". Traits are how Rust does polymorphism — both static
//! (generics, resolved at compile time) and dynamic (`dyn Trait`,
//! resolved through a vtable at runtime).

use std::fmt;
use std::ops::Add;

/// A trait with one required method and one default method. Types only
/// implement what the default cannot know; the default builds on the
/// required piece. This is the standard library's design everywhere
/// (Iterator requires only `next`; the other ~70 methods are defaults).
pub trait Describe {
    /// Required: every implementor must define this.
    fn name(&self) -> String;

    /// Default: implementors get it for free, but may override.
    fn description(&self) -> String {
        format!("<{}>", self.name())
    }
}

pub struct City {
    pub name: String,
    pub population: u32,
}

impl Describe for City {
    fn name(&self) -> String {
        self.name.clone()
    }

    // Override the default because we have more to say.
    fn description(&self) -> String {
        format!("{} (pop. {})", self.name, self.population)
    }
}

pub struct River {
    pub name: String,
}

impl Describe for River {
    fn name(&self) -> String {
        self.name.clone()
    }
    // No description override — the default kicks in.
}

/// Static dispatch: monomorphized per concrete type at compile time.
/// `impl Trait` in argument position is sugar for a generic parameter.
/// Zero runtime cost; the trade-off is one compiled copy per type used.
pub fn announce(item: &impl Describe) -> String {
    format!("now arriving: {}", item.description())
}

/// Dynamic dispatch: `dyn Describe` is a trait object — a (data ptr,
/// vtable ptr) pair. Needed when one collection must hold *different*
/// concrete types. Costs an indirect call; buys runtime heterogeneity.
pub fn roll_call(items: &[Box<dyn Describe>]) -> Vec<String> {
    items.iter().map(|item| item.name()).collect()
}

/// Implementing a standard trait: Display gives your type `{}`
/// formatting and a free `.to_string()`. Implement Display for
/// user-facing text; derive Debug for developer-facing text.
pub struct Temperature(pub f64);

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}°C", self.0)
    }
}

/// Operator overloading is just a trait implementation — `a + b`
/// desugars to `Add::add(a, b)`. Overload only when the operation is
/// genuinely the mathematical one; clever overloads confuse readers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

/// Trait bounds compose: this works for any T that can be compared
/// AND displayed. The `where` form reads better once bounds stack up.
pub fn larger_as_string<T>(a: T, b: T) -> String
where
    T: PartialOrd + fmt::Display,
{
    if a > b {
        a.to_string()
    } else {
        b.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_methods_and_overrides() {
        let city = City {
            name: "Oslo".into(),
            population: 700_000,
        };
        let river = River {
            name: "Rhine".into(),
        };

        assert_eq!(city.description(), "Oslo (pop. 700000)"); // overridden
        assert_eq!(river.description(), "<Rhine>"); // default
    }

    #[test]
    fn static_dispatch_accepts_any_implementor() {
        let river = River {
            name: "Donau".into(),
        };
        assert_eq!(announce(&river), "now arriving: <Donau>");
    }

    #[test]
    fn trait_objects_allow_mixed_collections() {
        // One Vec containing two different concrete types — only
        // possible through trait objects.
        let items: Vec<Box<dyn Describe>> = vec![
            Box::new(City {
                name: "Lyon".into(),
                population: 500_000,
            }),
            Box::new(River {
                name: "Seine".into(),
            }),
        ];
        assert_eq!(roll_call(&items), ["Lyon", "Seine"]);
    }

    #[test]
    fn display_gives_to_string_for_free() {
        assert_eq!(Temperature(21.55).to_string(), "21.6°C");
    }

    #[test]
    fn operators_are_traits() {
        let a = Vec2 { x: 1.0, y: 2.0 };
        let b = Vec2 { x: 3.0, y: 4.0 };
        assert_eq!(a + b, Vec2 { x: 4.0, y: 6.0 });
    }

    #[test]
    fn bounds_compose() {
        assert_eq!(larger_as_string(3, 7), "7");
        assert_eq!(larger_as_string("apple", "pear"), "pear");
    }
}

// Exercises
// ---------
// 1. Add a `Mountain` type implementing Describe; decide whether the
//    default description is good enough for it.
// 2. Implement `Sub` for Vec2, then `Mul<f64>` for scalar scaling —
//    note that Mul's type parameter lets `vec * 2.0` work.
// 3. Change `announce` to take `&dyn Describe` instead of
//    `&impl Describe`. What changes for callers? When would the dyn
//    version be required?
