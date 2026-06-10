//! Spawning threads, joining them, and moving data in.

use std::thread;

/// Spawn and join. `spawn` returns a JoinHandle; `join` blocks until
/// the thread finishes and returns its result. Forgetting to join is
/// a real bug: the program may exit while the thread is mid-work.
pub fn compute_in_background() -> u64 {
    let handle = thread::spawn(|| {
        // This closure runs on another OS thread.
        (1..=1_000u64).sum()
    });

    // join returns Result — Err means the thread panicked. Propagating
    // the panic with expect is right here: a panicked worker is a bug.
    handle.join().expect("worker thread panicked")
}

/// `move` closures: threads may outlive the function that spawned
/// them, so a spawned closure cannot BORROW local data — the borrow
/// checker rejects it:
///
/// ```compile_fail
/// use std::thread;
/// let numbers = vec![1, 2, 3];
/// thread::spawn(|| println!("{numbers:?}")); // error[E0373]: closure
/// // may outlive the current function, but it borrows `numbers`
/// ```
///
/// The fix is `move`: transfer ownership into the thread.
pub fn sum_owned_by_thread(numbers: Vec<i64>) -> i64 {
    let handle = thread::spawn(move || numbers.iter().sum());
    // `numbers` belongs to the thread now; using it here would not
    // compile. That is the data race prevention working.
    handle.join().expect("worker thread panicked")
}

/// Fan-out: split work across threads, join all, combine. Each thread
/// owns its chunk outright — cloning the chunks sidesteps lifetime
/// questions entirely, which is the right beginner default.
pub fn parallel_sum(numbers: &[i64], n_threads: usize) -> i64 {
    let n_threads = n_threads.max(1);
    let chunk_size = numbers.len().div_ceil(n_threads).max(1);

    let handles: Vec<_> = numbers
        .chunks(chunk_size)
        .map(|chunk| {
            let chunk = chunk.to_vec(); // own it, then move it
            thread::spawn(move || chunk.iter().sum::<i64>())
        })
        .collect();

    handles
        .into_iter()
        .map(|h| h.join().expect("worker thread panicked"))
        .sum()
}

/// Scoped threads (std since 1.63): threads guaranteed to finish
/// before the scope ends, so they may BORROW from the caller — no
/// clone, no move, no Arc. Use scoped threads whenever the parallelism
/// is bounded by a function body.
pub fn parallel_sum_borrowed(numbers: &[i64], n_threads: usize) -> i64 {
    let n_threads = n_threads.max(1);
    let chunk_size = numbers.len().div_ceil(n_threads).max(1);

    thread::scope(|scope| {
        let handles: Vec<_> = numbers
            .chunks(chunk_size)
            // No `move`, no clone: the scope proves the borrow is safe.
            .map(|chunk| scope.spawn(move || chunk.iter().sum::<i64>()))
            .collect();

        handles
            .into_iter()
            .map(|h| h.join().expect("worker thread panicked"))
            .sum()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_and_join() {
        assert_eq!(compute_in_background(), 500_500);
    }

    #[test]
    fn move_transfers_ownership_into_thread() {
        assert_eq!(sum_owned_by_thread(vec![1, 2, 3]), 6);
    }

    #[test]
    fn fan_out_matches_sequential_result() {
        let numbers: Vec<i64> = (1..=100).collect();
        let expected: i64 = numbers.iter().sum();
        assert_eq!(parallel_sum(&numbers, 4), expected);
        assert_eq!(parallel_sum(&numbers, 1), expected);
        // More threads than elements must still work.
        assert_eq!(parallel_sum(&[1, 2], 16), 3);
    }

    #[test]
    fn scoped_threads_borrow_instead_of_cloning() {
        let numbers: Vec<i64> = (1..=100).collect();
        assert_eq!(parallel_sum_borrowed(&numbers, 4), 5050);
    }
}

// Exercises
// ---------
// 1. Write a function that spawns one thread per word in a sentence,
//    each returning the word reversed, and reassembles the sentence in
//    the original order. (Order comes from joining in order.)
// 2. Benchmark parallel_sum vs a plain iterator sum for 10 million
//    elements. Where is the break-even point on your machine?
// 3. Rewrite parallel_sum using scoped threads, then explain why the
//    non-scoped version needed `to_vec`.
