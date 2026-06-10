//! Channels: message passing with `std::sync::mpsc`.
//!
//! "Do not communicate by sharing memory; share memory by
//! communicating." A channel moves OWNERSHIP of each message from
//! sender to receiver — after sending, the sender cannot touch the
//! value, so there is nothing to race on.

use std::sync::mpsc;
use std::thread;

/// One producer, one consumer. `recv` blocks until a message arrives;
/// iteration ends when every Sender is dropped — channel shutdown is
/// just ownership again.
pub fn pipeline() -> Vec<String> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for item in ["raw-1", "raw-2", "raw-3"] {
            // send moves the String into the channel. Using it after
            // send would not compile.
            tx.send(format!("processed {item}"))
                .expect("receiver alive");
        }
        // tx dropped here -> rx's iterator below terminates.
    });

    // Receiver implements IntoIterator: loop until channel closes.
    rx.into_iter().collect()
}

/// mpsc = Multi-Producer, Single-Consumer: clone the sender freely.
/// The receiver sees one merged stream. Arrival order across
/// producers is nondeterministic — never assert on it.
pub fn fan_in(n_workers: u32) -> Vec<u32> {
    let (tx, rx) = mpsc::channel();

    for id in 0..n_workers {
        let tx = tx.clone(); // each worker owns its own sender
        thread::spawn(move || {
            tx.send(id * 10).expect("receiver alive");
        });
    }
    // Drop the original sender — otherwise the receive loop never
    // ends, because ONE sender would still be alive. The most common
    // channel deadlock in practice.
    drop(tx);

    let mut results: Vec<u32> = rx.into_iter().collect();
    results.sort(); // impose order for the caller; arrival order is random
    results
}

/// Channels carry any Send type — structs with data, not just
/// primitives. Designing a small message enum is the channel
/// equivalent of designing a function signature.
#[derive(Debug, PartialEq)]
pub enum WorkerMessage {
    Progress { task_id: u32, percent: u8 },
    Done { task_id: u32 },
}

pub fn track_progress() -> Vec<WorkerMessage> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for percent in [50u8, 100] {
            tx.send(WorkerMessage::Progress {
                task_id: 7,
                percent,
            })
            .expect("receiver alive");
        }
        tx.send(WorkerMessage::Done { task_id: 7 })
            .expect("receiver alive");
    });

    rx.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_producer_preserves_order() {
        // One sender => FIFO order is guaranteed.
        assert_eq!(
            pipeline(),
            ["processed raw-1", "processed raw-2", "processed raw-3"]
        );
    }

    #[test]
    fn many_producers_merge_into_one_stream() {
        assert_eq!(fan_in(4), [0, 10, 20, 30]);
        assert!(fan_in(0).is_empty());
    }

    #[test]
    fn structured_messages() {
        let messages = track_progress();
        assert_eq!(messages.len(), 3);
        // Single sender, so order IS deterministic here.
        assert_eq!(messages[2], WorkerMessage::Done { task_id: 7 });
    }
}

// Exercises
// ---------
// 1. Build a worker pool: N threads pulling jobs from one channel and
//    pushing results into another. (Hint: the JOB channel's Receiver
//    is single-consumer — wrap it in Arc<Mutex<Receiver>> so workers
//    can share it. See shared_state.rs.)
// 2. What happens if you remove the `drop(tx)` in fan_in? Predict,
//    then try it. (cargo test will hang — ctrl-c is part of the
//    lesson.)
// 3. Use `sync_channel(0)` to build a rendezvous: sender blocks until
//    the receiver is ready. When is backpressure like this useful?
