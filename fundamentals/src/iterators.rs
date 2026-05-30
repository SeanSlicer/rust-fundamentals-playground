//! Iterators: Rust's most-used abstraction.
//!
//! Iterators are lazy — adapters like `map` and `filter` do nothing
//! until a consumer (`collect`, `sum`, `for`...) drives them. Chains
//! of adapters compile down to the same machine code as hand-written
//! loops, so prefer them: clearer intent at zero cost.

/// The three ways to get an iterator from a collection — choosing the
/// wrong one is the most common iterator mistake:
/// * `iter()`       borrows: yields `&T`, collection stays usable.
/// * `iter_mut()`   borrows mutably: yields `&mut T` for in-place edits.
/// * `into_iter()`  consumes: yields `T`, collection is gone afterward.
// Clippy suggests arrays where these vecs are only iterated — but the
// whole point is iterating COLLECTIONS three different ways.
#[allow(clippy::useless_vec)]
pub fn three_kinds_of_iteration() -> (i32, Vec<i32>, Vec<String>) {
    let v = vec![1, 2, 3];
    let total: i32 = v.iter().sum(); // &i32s, v still owned by us

    let mut w = vec![1, 2, 3];
    w.iter_mut().for_each(|n| *n += 10); // mutate through &mut i32

    let names = vec![String::from("a"), String::from("b")];
    let owned: Vec<String> = names.into_iter().collect(); // names is moved
                                                          // `names` is unusable from here on — uncomment to see E0382:
                                                          // println!("{names:?}");

    (total, w, owned)
}

/// A typical adapter chain: filter, transform, collect. Read it as a
/// data pipeline. `filter_map` fuses the two middle steps when the
/// transform itself can reject items.
pub fn even_squares(numbers: &[i32]) -> Vec<i32> {
    numbers
        .iter()
        .filter(|&&n| n % 2 == 0) // keep evens (note: yields &&i32)
        .map(|&n| n * n) // square them
        .collect() // drive the lazy chain, build a Vec
}

/// Parse the valid numbers out of messy input — `filter_map` keeps the
/// Somes and drops the Nones in one pass. This is THE idiom for
/// "best-effort" parsing.
pub fn parse_valid_numbers(words: &[&str]) -> Vec<i32> {
    words.iter().filter_map(|w| w.parse().ok()).collect()
}

/// collect() into Result: if every element parses, you get
/// Ok(Vec<i32>); the FIRST failure aborts and returns that Err. Use
/// this when partial success is not acceptable.
pub fn parse_all_numbers(words: &[&str]) -> Result<Vec<i32>, std::num::ParseIntError> {
    words.iter().map(|w| w.parse()).collect()
}

/// zip + enumerate: combine parallel sequences and number items
/// without manual index bookkeeping.
pub fn rank_scores(names: &[&str], scores: &[u32]) -> Vec<String> {
    names
        .iter()
        .zip(scores) // pairs (name, score), stops at the shorter side
        .enumerate() // adds a 0-based position
        .map(|(i, (name, score))| format!("{}. {name} ({score})", i + 1))
        .collect()
}

/// fold: the general-purpose consumer everything else is built from.
/// Reach for a named consumer (sum, max, count...) first; use fold
/// when no named one fits.
pub fn longest_word_length(text: &str) -> usize {
    text.split_whitespace()
        .fold(0, |longest, word| longest.max(word.chars().count()))
}

// ---------------------------------------------------------------------------
// Implementing your own iterator
// ---------------------------------------------------------------------------
// One method is all it takes: `next` returns Some(item) until the
// sequence ends, then None. Every adapter and consumer above works on
// your type for free.

/// Yields the Fibonacci sequence forever. Infinite iterators are fine
/// because of laziness — callers bound them with `take`.
pub struct Fibonacci {
    current: u64,
    next: u64,
}

impl Fibonacci {
    pub fn new() -> Self {
        Fibonacci {
            current: 0,
            next: 1,
        }
    }
}

impl Default for Fibonacci {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let result = self.current;
        // Advance the pair; this is the whole state machine.
        self.next += self.current;
        self.current = self.next - self.current;
        Some(result) // never None: infinite sequence
    }
}

/// The payoff: our iterator composes with the std adapters.
pub fn even_fibonacci_below(limit: u64) -> Vec<u64> {
    Fibonacci::new()
        .take_while(|&n| n < limit)
        .filter(|n| n % 2 == 0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iteration_kinds() {
        let (total, mutated, owned) = three_kinds_of_iteration();
        assert_eq!(total, 6);
        assert_eq!(mutated, vec![11, 12, 13]);
        assert_eq!(owned, vec!["a", "b"]);
    }

    #[test]
    fn adapter_chains() {
        assert_eq!(even_squares(&[1, 2, 3, 4]), vec![4, 16]);
        assert!(even_squares(&[1, 3]).is_empty());
    }

    #[test]
    fn best_effort_vs_all_or_nothing_parsing() {
        let input = ["1", "two", "3"];
        assert_eq!(parse_valid_numbers(&input), vec![1, 3]);
        assert!(parse_all_numbers(&input).is_err());
        assert_eq!(parse_all_numbers(&["1", "3"]), Ok(vec![1, 3]));
    }

    #[test]
    fn zip_and_enumerate() {
        let ranked = rank_scores(&["ana", "bob"], &[90, 80]);
        assert_eq!(ranked, ["1. ana (90)", "2. bob (80)"]);
    }

    #[test]
    fn fold_when_no_named_consumer_fits() {
        assert_eq!(longest_word_length("a bb ccc"), 3);
        assert_eq!(longest_word_length(""), 0);
    }

    #[test]
    fn custom_iterator_composes_with_adapters() {
        let fib: Vec<u64> = Fibonacci::new().take(7).collect();
        assert_eq!(fib, vec![0, 1, 1, 2, 3, 5, 8]);
        assert_eq!(even_fibonacci_below(100), vec![0, 2, 8, 34]);
    }
}

// Exercises
// ---------
// 1. Rewrite `parse_valid_numbers` with an explicit for loop. Count
//    the lines, then decide which version you'd want to maintain.
// 2. Implement a `Countdown` iterator that yields n, n-1, ..., 1.
//    Make `Countdown::new(3).sum::<u32>()` return 6.
// 3. Use `Fibonacci` with `position` to find the index of the first
//    Fibonacci number above 1000.
// 4. Investigate why `filter(|&&n| ...)` needs two ampersands here —
//    what does `iter()` yield, and what does filter pass its closure?
