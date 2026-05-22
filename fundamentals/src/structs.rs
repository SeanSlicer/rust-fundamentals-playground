//! Structs: named product types, methods, and associated functions.
//!
//! Rust separates data (the struct) from behavior (`impl` blocks).
//! There is no inheritance — composition and traits (see the
//! intermediate crate) fill that role.

/// A classic named-field struct. `#[derive(...)]` asks the compiler to
/// generate common trait implementations: `Debug` for `{:?}` printing,
/// `PartialEq` for `==`, `Clone` for explicit copying.
#[derive(Debug, Clone, PartialEq)]
pub struct Rectangle {
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    /// An *associated function* (no `self`) — Rust's idiom for
    /// constructors. There is nothing magic about the name `new`;
    /// it is just convention.
    pub fn new(width: u32, height: u32) -> Self {
        // Field init shorthand: `width` instead of `width: width`.
        Self { width, height }
    }

    /// A second constructor. Multiple named constructors replace
    /// constructor overloading from other languages.
    pub fn square(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }

    /// `&self` borrows the struct immutably — the method only reads.
    /// Choosing the right self type (`&self`, `&mut self`, `self`) is
    /// part of the API contract.
    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    /// `&mut self` because we modify the struct in place.
    pub fn scale(&mut self, factor: u32) {
        self.width *= factor;
        self.height *= factor;
    }

    /// Taking `self` by value consumes the rectangle and produces a
    /// new one — the builder/transform pattern. The caller cannot use
    /// the original afterwards, which makes "this replaces the old
    /// value" explicit in the type system.
    pub fn rotated(self) -> Self {
        Self {
            width: self.height,
            height: self.width,
        }
    }

    pub fn can_hold(&self, other: &Rectangle) -> bool {
        self.width >= other.width && self.height >= other.height
    }
}

/// Tuple structs: a name for a tuple. Most useful as "newtypes" — a
/// zero-cost wrapper giving a primitive a distinct type so you cannot
/// accidentally mix up, say, meters and seconds.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Meters(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Seconds(pub f64);

/// The newtype pattern at work: this signature cannot be called with
/// the arguments swapped, unlike `fn speed(d: f64, t: f64)`.
pub fn speed(distance: Meters, time: Seconds) -> f64 {
    distance.0 / time.0
}

/// Struct update syntax: build a new struct from an old one, changing
/// only some fields. Note this *moves* non-Copy fields out of `base`.
#[derive(Debug, Clone, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub verbose: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            port: 8080,
            verbose: false,
        }
    }
}

pub fn config_on_port(port: u16) -> ServerConfig {
    ServerConfig {
        port,
        // Take every other field from the default. `..` must come last.
        ..ServerConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_and_methods() {
        let r = Rectangle::new(3, 4);
        assert_eq!(r.area(), 12);
        assert_eq!(Rectangle::square(5).area(), 25);
    }

    #[test]
    fn mutating_methods_need_mut_binding() {
        let mut r = Rectangle::new(2, 3);
        r.scale(2);
        assert_eq!(r, Rectangle::new(4, 6));
    }

    #[test]
    fn consuming_methods_produce_new_values() {
        let r = Rectangle::new(1, 2).rotated();
        assert_eq!(r, Rectangle::new(2, 1));
    }

    #[test]
    fn can_hold_compares_dimensions() {
        let big = Rectangle::new(10, 10);
        let small = Rectangle::new(3, 3);
        assert!(big.can_hold(&small));
        assert!(!small.can_hold(&big));
    }

    #[test]
    fn newtypes_prevent_argument_mixups() {
        let v = speed(Meters(100.0), Seconds(9.58));
        assert!(v > 10.0 && v < 11.0);
    }

    #[test]
    fn struct_update_syntax_fills_in_the_rest() {
        let config = config_on_port(9999);
        assert_eq!(config.port, 9999);
        assert_eq!(config.host, "localhost"); // came from default()
    }
}

// Exercises
// ---------
// 1. Add a `perimeter` method to Rectangle and a unit test for it.
// 2. Create a `Celsius` newtype and a conversion to a `Fahrenheit`
//    newtype. Make the conversion a method, then try implementing the
//    standard `From` trait instead.
// 3. Give ServerConfig a builder: `ServerConfig::builder().port(80)
//    .verbose(true).build()`. Decide whether build() should consume
//    the builder.
