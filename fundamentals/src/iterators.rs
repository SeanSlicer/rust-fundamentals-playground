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

