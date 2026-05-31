//! Lifetimes: telling the compiler how long references must stay
//! valid.
//!
//! Lifetimes don't change how long anything lives — they *describe*
//! relationships between references so the compiler can verify none
//! outlives its data. Most lifetimes are inferred (elision); you write
//! them only when the relationship is ambiguous.

/// The canonical case where annotation is required. The compiler
/// cannot know whether the returned reference borrows from `a` or `b`
/// — it depends on runtime data. `'a` says: "the result lives no
/// longer than the SHORTER of the two inputs". That is a promise the
/// caller must respect.
pub fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() {
        a
    } else {
        b
    }
}

/// With two reference parameters, elision cannot decide which one the
/// output borrows from, so we annotate. Being precise pays off: tying
/// the result only to `s` (and not to `prefix`) tells the compiler
/// that `prefix` may be a short-lived temporary — see the test below,
/// which would not compile if both parameters shared `'a`.
pub fn trim_prefix<'a>(s: &'a str, prefix: &str) -> &'a str {
    s.strip_prefix(prefix).unwrap_or(s)
}

/// Structs holding references need lifetime parameters: the struct
/// must not outlive what it borrows. Use this for cheap, short-lived
/// "view" types; if the struct needs to live independently, store
/// owned data (String) instead.
#[derive(Debug)]
pub struct Excerpt<'a> {
    pub text: &'a str,
}

impl<'a> Excerpt<'a> {
    /// Borrow the first sentence of a larger text. No allocation —
    /// the Excerpt is a window into `source`.
    pub fn first_sentence(source: &'a str) -> Self {
        let end = source.find('.').map(|i| i + 1).unwrap_or(source.len());
        Excerpt {
            text: &source[..end],
        }
    }

    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }
}

/// `'static` means "lives for the entire program". String literals are
/// 'static because they are baked into the binary. Don't reach for
/// 'static to silence the borrow checker — it is usually the wrong
/// fix; owning the data is the right one.
pub fn motto() -> &'static str {
    "fast, reliable, productive — pick three"
}

/// The dangling-reference error lifetimes prevent, for reference:
///
/// ```compile_fail
/// let result;
/// {
///     let short_lived = String::from("hello");
///     result = intermediate::lifetimes::longest(short_lived.as_str(), "hi");
/// } // short_lived dropped here...
/// println!("{result}"); // error[E0597]: borrowed value does not live long enough
/// ```
///
/// The signature of `longest` is what makes this *checkable*: result
/// may not outlive the shorter-lived argument.
pub fn lifetimes_are_promises() -> &'static str {
    "the compile_fail example above is verified by cargo test"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_picks_by_length() {
        assert_eq!(longest("hello", "hi"), "hello");
        // Result borrows from the arguments — both alive here, fine.
        let a = String::from("short");
        let b = String::from("looonger");
        assert_eq!(longest(&a, &b), "looonger");
    }

    #[test]
    fn precise_lifetimes_allow_more_callers() {
        // `prefix` can be a temporary BECAUSE the signature says the
        // result never borrows from it.
        let s = String::from("rust-lang");
        let trimmed = trim_prefix(&s, &format!("rust{}", "-"));
        assert_eq!(trimmed, "lang"); // temporary prefix already gone — ok!
    }

    #[test]
    fn view_structs_borrow_without_allocating() {
        let article = String::from("Rust is fast. It is also safe.");
        let excerpt = Excerpt::first_sentence(&article);
        assert_eq!(excerpt.text, "Rust is fast.");
        assert_eq!(excerpt.word_count(), 3);
        // `article` must outlive `excerpt` — the compiler enforces it.
    }

    #[test]
    fn static_lifetime() {
        let m: &'static str = motto();
        assert!(m.contains("pick three"));
    }
}

// Exercises
// ---------
// 1. Write `fn first_and_last<'a>(s: &'a str) -> (&'a str, &'a str)`
//    returning the first and last whitespace-separated words.
// 2. Try changing `longest` to return `&'a str` where 'a is only on
//    parameter `a`. Which call sites break, and why is the compiler
//    right to break them?
// 3. Give Excerpt a method `extend_to_second_sentence(&self, source:
//    &'a str) -> Excerpt<'a>` and think about why the parameter
//    lifetime must be 'a, not a fresh one.
