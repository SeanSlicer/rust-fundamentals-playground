//! Shared state: Arc<Mutex<T>> and RwLock.
//!
//! When message passing doesn't fit (e.g. a shared cache or counter),
//! threads share data directly — but Rust insists on the full recipe:
//! * `Arc`   — atomically reference-counted ownership across threads
//!   (Rc is NOT Send; the compiler rejects it in spawn).
//! * `Mutex` — at most one thread touches the data at a time.
//!
//! Neither half is optional: Arc without Mutex gives shared READ-ONLY
//! access; Mutex without Arc can't be given to a second thread.

use std::sync::{Arc, Mutex, RwLock};
use std::thread;

/// The canonical example: many threads incrementing one counter.
/// Without the Mutex this is a data race — and, crucially, it is a
/// COMPILE ERROR, not a heisenbug.
pub fn parallel_counter(n_threads: u32, increments_per_thread: u32) -> u32 {
    let counter = Arc::new(Mutex::new(0u32));

    let handles: Vec<_> = (0..n_threads)
        .map(|_| {
            // Each thread gets its own Arc handle (count +1), all
            // pointing at the same Mutex.
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..increments_per_thread {
                    // lock() blocks until the mutex is free. The
                    // returned guard derefs to the data...
                    let mut value = counter.lock().expect("mutex not poisoned");
                    *value += 1;
                } // ...and unlocks HERE, when the guard drops. There
                  // is no unlock() to forget — RAII does it.
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("worker thread panicked");
    }

    let result = *counter.lock().expect("mutex not poisoned");
    result
}

/// Keep critical sections small: compute outside the lock, mutate
/// inside it. Holding a lock during slow work serializes every other
/// thread — correctness-adjacent, but the #1 practical mutex mistake.
pub fn append_results(inputs: &[i64]) -> Vec<i64> {
    let results = Arc::new(Mutex::new(Vec::new()));

    thread::scope(|scope| {
        for &input in inputs {
            let results = Arc::clone(&results);
            scope.spawn(move || {
                // "Expensive" computation happens WITHOUT the lock...
                let computed = input * input;
                // ...the lock is held only for the cheap push.
                results.lock().expect("mutex not poisoned").push(computed);
            });
        }
    });

    let mut out = Arc::try_unwrap(results)
        .expect("all threads joined, single owner remains")
        .into_inner()
        .expect("mutex not poisoned");
    out.sort(); // thread completion order is nondeterministic
    out
}

/// RwLock: many concurrent readers OR one writer — the borrow rules
/// as a runtime lock. Prefer it over Mutex when reads vastly
/// outnumber writes; otherwise Mutex is simpler and often faster.
pub struct SettingsStore {
    settings: RwLock<Vec<(String, String)>>,
}

impl SettingsStore {
    pub fn new() -> Self {
        SettingsStore {
            settings: RwLock::new(Vec::new()),
        }
    }

    pub fn set(&self, key: &str, value: &str) {
        let mut guard = self.settings.write().expect("lock not poisoned");
        match guard.iter_mut().find(|(k, _)| k == key) {
            Some((_, v)) => *v = value.to_string(),
            None => guard.push((key.to_string(), value.to_string())),
        }
    }

    /// Multiple threads can run `get` simultaneously — read locks
    /// don't exclude each other.
    pub fn get(&self, key: &str) -> Option<String> {
        self.settings
            .read()
            .expect("lock not poisoned")
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }
}

impl Default for SettingsStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_lost_updates() {
        // 8 threads x 1000 increments: without the mutex, lost updates
        // would make this flaky; with it, the count is exact.
        assert_eq!(parallel_counter(8, 1000), 8000);
    }

    #[test]
    fn results_collected_from_all_threads() {
        assert_eq!(append_results(&[1, 2, 3]), vec![1, 4, 9]);
    }

    #[test]
    fn rwlock_settings_store() {
        let store = SettingsStore::new();
        store.set("theme", "dark");
        store.set("theme", "light"); // overwrite
        assert_eq!(store.get("theme"), Some("light".into()));
        assert_eq!(store.get("missing"), None);
    }

    #[test]
    fn concurrent_readers_share_the_store() {
        let store = Arc::new(SettingsStore::new());
        store.set("lang", "rust");

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let store = Arc::clone(&store);
                thread::spawn(move || store.get("lang"))
            })
            .collect();

        for handle in handles {
            assert_eq!(handle.join().unwrap(), Some("rust".into()));
        }
    }
}

// Exercises
// ---------
// 1. Replace the Mutex in parallel_counter with
//    std::sync::atomic::AtomicU32 — no lock at all. When are atomics
//    enough, and when do you really need a Mutex?
// 2. Try `Rc<Mutex<u32>>` in parallel_counter instead of Arc. Read the
//    full compile error: which trait is missing, and on which type?
// 3. Deadlock by hand: two Mutexes, two threads, opposite lock order.
//    Then fix it by agreeing on a global lock order.
