//! Interior mutability: mutating data behind a shared reference.
//!
//! Normally `&T` means "read only". Cell and RefCell carve out a
//! controlled exception: the VALUE can change even though the BINDING
//! is shared. The borrow rules don't disappear — RefCell moves the
//! check from compile time to runtime (and panics on violation).
//!
//! When you need it: shared ownership that also needs mutation
//! (Rc<RefCell<T>>), caches and counters inside logically-immutable
//! types, mock objects recording calls through &self.

use std::cell::{Cell, RefCell};

/// Cell<T>: for Copy types. No references into the cell are ever
/// given out — you `get` a copy and `set` a new value — so there is
/// nothing to check and it can never panic.
pub struct HitCounter {
    hits: Cell<u32>,
}

impl HitCounter {
    pub fn new() -> Self {
        HitCounter { hits: Cell::new(0) }
    }

    /// Note: &self, not &mut self. The whole point — callers holding
    /// only a shared reference can still record a hit. Without Cell,
    /// this method would need &mut self and infect every caller with
    /// mutability requirements.
    pub fn record(&self) {
        self.hits.set(self.hits.get() + 1);
    }

    pub fn count(&self) -> u32 {
        self.hits.get()
    }
}

impl Default for HitCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// RefCell<T>: for non-Copy types. `borrow()` and `borrow_mut()`
/// enforce the usual rules (many readers XOR one writer) at RUNTIME.
/// A violation is a panic, not a compile error — the trade-off for
/// flexibility the compiler cannot verify.
pub struct EventLog {
    entries: RefCell<Vec<String>>,
}

impl EventLog {
    pub fn new() -> Self {
        EventLog {
            entries: RefCell::new(Vec::new()),
        }
    }

    /// Again &self: a logger that required &mut self would be
    /// unusable — everything touches the logger.
    pub fn log(&self, message: &str) {
        self.entries.borrow_mut().push(message.to_string());
    }

    pub fn len(&self) -> usize {
        self.entries.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.borrow().is_empty()
    }

    /// Keep borrows SHORT. Cloning out is often the pragmatic choice;
    /// holding a Ref across unrelated code invites runtime panics.
    pub fn snapshot(&self) -> Vec<String> {
        self.entries.borrow().clone()
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}

/// The classic runtime failure: two overlapping mutable borrows.
/// The compile-time rules did not vanish — they just fire later.
pub fn double_borrow_panics() {
    let cell = RefCell::new(vec![1, 2, 3]);
    let _first = cell.borrow_mut();
    let _second = cell.borrow_mut(); // panics: already mutably borrowed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_mutates_through_shared_ref() {
        let counter = HitCounter::new();
        let r1 = &counter;
        let r2 = &counter; // two shared refs, both can record
        r1.record();
        r2.record();
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn refcell_for_non_copy_data() {
        let log = EventLog::new();
        log.log("started");
        log.log("finished");
        assert_eq!(log.len(), 2);
        assert_eq!(log.snapshot(), ["started", "finished"]);
    }

    #[test]
    #[should_panic(expected = "already")]
    fn refcell_violations_panic_at_runtime() {
        // should_panic documents the failure mode as a test — the
        // runtime equivalent of a compile_fail doctest.
        double_borrow_panics();
    }
}

// Exercises
// ---------
// 1. Build `Rc<RefCell<Vec<i32>>>` shared by two owners; push from
//    one, read from the other. This combo is the workhorse of
//    single-threaded shared mutable state.
// 2. Add a `clear` method to EventLog and a test proving it works
//    through a shared reference.
// 3. Why is Cell limited to Copy types? What could go wrong if you
//    could `get` a String out of a Cell while the cell still held it?
