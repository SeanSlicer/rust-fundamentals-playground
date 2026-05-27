//! The standard collections: Vec, HashMap, HashSet, VecDeque.
//!
//! Rule of thumb: use Vec until you have a reason not to. HashMap when
//! you look things up by key, HashSet when you only care about
//! membership, VecDeque when you push/pop at both ends.

use std::collections::{HashMap, HashSet, VecDeque};

/// Vec basics: grow, index, iterate. Indexing panics out of bounds;
/// `get` returns Option (see the slices module).
pub fn build_squares(n: u32) -> Vec<u32> {
    // With a known size, `with_capacity` avoids reallocation. This is
    // an optimization, not a correctness issue — plain Vec::new() is
    // fine until profiling says otherwise.
    let mut squares = Vec::with_capacity(n as usize);
    for i in 1..=n {
        squares.push(i * i);
    }
    squares
}

/// HashMap: the `entry` API is the idiomatic way to "insert or
/// update". The naive version (contains_key then insert) does two
/// lookups and is racy in spirit; `entry` does one lookup.
pub fn word_counts(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        // or_insert returns &mut usize — we update through it.
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

/// Ownership and HashMap: `get` borrows; `remove` moves the value out.
/// Keys of owned types (String) are moved in on insert.
pub fn pop_translation(dict: &mut HashMap<String, String>, word: &str) -> Option<String> {
    // remove gives back ownership of the value — useful when you want
    // to consume the stored data, not just look at it.
    dict.remove(word)
}

/// HashSet: membership testing and deduplication. Set operations
/// (union, intersection, difference) come for free.
pub fn shared_letters(a: &str, b: &str) -> HashSet<char> {
    let set_a: HashSet<char> = a.chars().collect();
    let set_b: HashSet<char> = b.chars().collect();
    // intersection yields &char; copied() turns them into char.
    set_a.intersection(&set_b).copied().collect()
}

/// VecDeque: a ring buffer. O(1) push/pop at both ends, which Vec
/// cannot do (Vec's remove(0) shifts every element). The natural fit
/// for queues and sliding windows.
pub fn last_n_events(events: &[&str], n: usize) -> VecDeque<String> {
    let mut window = VecDeque::with_capacity(n);
    for event in events {
        if window.len() == n {
            window.pop_front(); // evict the oldest
        }
        window.push_back(event.to_string());
    }
    window
}

/// Choosing by access pattern, demonstrated: top-k frequent words.
/// HashMap for counting, Vec for sorting — collections compose.
pub fn top_words(text: &str, k: usize) -> Vec<(String, usize)> {
    let counts = word_counts(text);
    let mut pairs: Vec<(String, usize)> = counts.into_iter().collect();
    // Sort by count descending, then alphabetically for a stable,
    // testable order.
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    pairs.truncate(k);
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_growth() {
        assert_eq!(build_squares(4), vec![1, 4, 9, 16]);
        assert!(build_squares(0).is_empty());
    }

    #[test]
    fn entry_api_counts_words() {
        let counts = word_counts("the cat and the hat");
        assert_eq!(counts.get("the"), Some(&2));
        assert_eq!(counts.get("cat"), Some(&1));
        assert_eq!(counts.get("dog"), None);
    }

    #[test]
    fn remove_transfers_ownership_out() {
        let mut dict = HashMap::new();
        dict.insert("hello".to_string(), "hola".to_string());
        assert_eq!(pop_translation(&mut dict, "hello"), Some("hola".into()));
        assert_eq!(pop_translation(&mut dict, "hello"), None); // gone now
    }

    #[test]
    fn set_intersection() {
        let shared = shared_letters("rust", "trust");
        let expected: HashSet<char> = "rust".chars().collect();
        assert_eq!(shared, expected);
    }

    #[test]
    fn deque_keeps_a_sliding_window() {
        let window = last_n_events(&["a", "b", "c", "d"], 2);
        assert_eq!(window, ["c", "d"]);
    }

    #[test]
    fn collections_compose() {
        let top = top_words("a b b c c c", 2);
        assert_eq!(top, vec![("c".to_string(), 3), ("b".to_string(), 2)]);
    }
}

// Exercises
// ---------
// 1. Group a list of words by their first letter into a
//    HashMap<char, Vec<String>> using the entry API.
// 2. Implement a queue with Vec instead of VecDeque and explain (in a
//    comment) why pop-from-front is O(n) there.
// 3. Given two Vec<i32>, return the elements unique to each — pick the
//    right set operation.
