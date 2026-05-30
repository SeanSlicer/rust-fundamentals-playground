//! Generics: write the logic once, let the compiler stamp out a
//! version per type.
//!
//! Generics are resolved at compile time (monomorphization): using
//! `largest` with i32 and with &str produces two specialized
//! functions, each as fast as if you had written it by hand. The cost
//! is compile time and binary size, not runtime speed.

/// A generic function with a trait bound. Without `PartialOrd` the
/// body could not use `>` — bounds declare exactly what the function
/// needs from T, nothing more. Taking and returning references avoids
/// requiring `Copy` or `Clone`.
pub fn largest<T: PartialOrd>(items: &[T]) -> Option<&T> {
    let mut iter = items.iter();
    let mut largest = iter.next()?; // empty slice -> None, not panic
    for item in iter {
        if item > largest {
            largest = item;
        }
    }
    Some(largest)
}

/// A generic struct. One definition, any payload type.
#[derive(Debug, Clone, PartialEq)]
pub struct Pair<T> {
    pub first: T,
    pub second: T,
}

/// Methods available for every T.
impl<T> Pair<T> {
    pub fn new(first: T, second: T) -> Self {
        Self { first, second }
    }

    pub fn swap(self) -> Self {
        Self {
            first: self.second,
            second: self.first,
        }
    }
}

/// Methods available only when T meets extra bounds — conditional
/// implementation. `Pair<File>` simply does not have `larger`; the
/// method appears only for comparable types. This is how std gives
/// `Vec<T>` a `sort` method only when T: Ord.
impl<T: PartialOrd + Copy> Pair<T> {
    pub fn larger(&self) -> T {
        if self.first > self.second {
            self.first
        } else {
            self.second
        }
    }
}

/// Multiple type parameters. U and T can be the same type or
/// different; the compiler infers both from the call site.
pub fn map_pair<T, U>(pair: Pair<T>, f: impl Fn(T) -> U) -> Pair<U> {
    Pair {
        first: f(pair.first),
        second: f(pair.second),
    }
}

/// Generic enums — you already use them daily: Option<T> and
/// Result<T, E>. Here is a domain-flavored one.
#[derive(Debug, PartialEq)]
pub enum Measurement<T> {
    Exact(T),
    Approximate { value: T, error_margin: T },
}

impl<T: Copy> Measurement<T> {
    pub fn value(&self) -> T {
        match self {
            Measurement::Exact(v) => *v,
            Measurement::Approximate { value, .. } => *value,
        }
    }
}

/// Turbofish syntax `::<>` supplies type arguments explicitly when
/// inference has nothing to go on. Most common with collect and parse.
pub fn parse_with_turbofish(s: &str) -> Option<u16> {
    s.parse::<u16>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_function_many_types() {
        assert_eq!(largest(&[1, 5, 3]), Some(&5));
        assert_eq!(largest(&["pear", "apple"]), Some(&"pear"));
        assert_eq!(largest::<i32>(&[]), None);
    }

    #[test]
    fn generic_structs() {
        let p = Pair::new(1, 2).swap();
        assert_eq!(p, Pair::new(2, 1));
        let words = Pair::new("a".to_string(), "b".to_string());
        assert_eq!(words.first, "a"); // works for non-Copy types too
    }

    #[test]
    fn conditional_methods_exist_only_with_bounds() {
        assert_eq!(Pair::new(3, 9).larger(), 9);
        // Pair::new(vec![1], vec![2]).larger() would not compile:
        // Vec is neither PartialOrd-by-Copy nor Copy.
    }

    #[test]
    fn mapping_changes_the_type_parameter() {
        let lengths = map_pair(Pair::new("hi".to_string(), "there".to_string()), |s| {
            s.len()
        });
        assert_eq!(lengths, Pair::new(2, 5));
    }

    #[test]
    fn generic_enums() {
        let exact = Measurement::Exact(42);
        let rough = Measurement::Approximate {
            value: 40,
            error_margin: 5,
        };
        assert_eq!(exact.value(), 42);
        assert_eq!(rough.value(), 40);
    }

    #[test]
    fn turbofish() {
        assert_eq!(parse_with_turbofish("80"), Some(80));
        assert_eq!(parse_with_turbofish("70000"), None); // overflows u16
    }
}

// Exercises
// ---------
// 1. Write `fn smallest<T: PartialOrd>(items: &[T]) -> Option<&T>`
//    without copying `largest`'s body — can both delegate to one
//    helper taking a comparison closure?
// 2. Give Measurement a `map` method like Option's, converting
//    Measurement<T> to Measurement<U>.
// 3. Why does `largest` return Option<&T> rather than Option<T>?
//    Change it and see which extra bound the compiler demands.
