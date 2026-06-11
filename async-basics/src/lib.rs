//! Async basics with Tokio.
//!
//! The model in three sentences: `async fn` returns a Future — a
//! paused computation that does nothing until polled. `.await` yields
//! control to the runtime while waiting, so ONE thread can interleave
//! thousands of waiting tasks. An executor (Tokio here — std provides
//! the syntax but no runtime) does the polling.
//!
//! Async vs threads: async shines for I/O-bound work with many
//! concurrent waits (sockets, timers). For CPU-bound work, threads
//! (see the `concurrency` crate) are the right tool — an async task
//! that computes for 100ms blocks every other task on its thread.

use std::time::Duration;
use tokio::time::sleep;

/// An async function. Calling it does NOT run the body — it builds a
/// Future. The body runs (in pieces) as the runtime polls it past
/// each await point.
pub async fn fetch_setting(key: &str) -> String {
    // Simulated I/O latency. tokio::time::sleep yields to the runtime;
    // std::thread::sleep would BLOCK the whole worker thread — the
    // classic async bug.
    sleep(Duration::from_millis(10)).await;
    format!("{key}=enabled")
}

/// Sequential awaits: the second fetch starts only after the first
/// finishes. Total time ~ sum of latencies. Correct, but often not
/// what you want.
pub async fn fetch_two_sequential() -> (String, String) {
    let a = fetch_setting("alpha").await;
    let b = fetch_setting("beta").await;
    (a, b)
}

/// Concurrent awaits with join!: both futures progress at once on the
/// same task; total time ~ max of latencies. This is the async win,
/// and it needs no extra threads.
pub async fn fetch_two_concurrent() -> (String, String) {
    let (a, b) = tokio::join!(fetch_setting("alpha"), fetch_setting("beta"));
    (a, b)
}

/// tokio::spawn: hand a future to the runtime as an independent TASK
/// that runs even if nobody awaits it immediately — async's version of
/// thread::spawn, but tasks cost ~hundreds of bytes, not megabytes of
/// stack. The future must be 'static (own its data): same rule, same
/// reason as `move` closures with threads.
pub async fn spawn_workers(n: u32) -> Vec<String> {
    let handles: Vec<_> = (0..n)
        .map(|i| tokio::spawn(async move { fetch_setting(&format!("worker-{i}")).await }))
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        // JoinHandle is itself a future; Err = task panicked.
        results.push(handle.await.expect("task panicked"));
    }
    results
}

/// Async channels mirror std::sync::mpsc, but send/recv are awaits,
/// not blocking calls — while a receiver waits, the thread serves
/// other tasks. The bounded capacity provides backpressure: senders
/// await when the buffer is full.
pub async fn producer_consumer() -> Vec<u32> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(2);

    tokio::spawn(async move {
        for i in 1..=5u32 {
            tx.send(i * 100).await.expect("receiver alive");
        }
        // tx drops -> rx.recv() starts returning None.
    });

    let mut received = Vec::new();
    while let Some(value) = rx.recv().await {
        received.push(value);
    }
    received
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test] wraps the async test in a runtime — the test
    // equivalent of #[tokio::main].
    #[tokio::test]
    async fn futures_run_when_awaited() {
        assert_eq!(fetch_setting("cache").await, "cache=enabled");
    }

    // start_paused: the runtime fakes the clock and auto-advances it
    // whenever every task is sleeping. Timing assertions become exact
    // and instant — the only sane way to test time-dependent async
    // code (a wall-clock assertion would be flaky on a busy machine).
    #[tokio::test(start_paused = true)]
    async fn join_runs_futures_concurrently() {
        let start = tokio::time::Instant::now();
        let (a, b) = fetch_two_concurrent().await;
        let elapsed = start.elapsed();

        assert_eq!(a, "alpha=enabled");
        assert_eq!(b, "beta=enabled");
        // Two 10ms waits overlapped: exactly 10ms of virtual time,
        // not 20.
        assert_eq!(elapsed, Duration::from_millis(10), "took {elapsed:?}");
    }

    #[tokio::test(start_paused = true)]
    async fn sequential_awaits_add_up() {
        let start = tokio::time::Instant::now();
        let (a, b) = fetch_two_sequential().await;

        assert_eq!((a.as_str(), b.as_str()), ("alpha=enabled", "beta=enabled"));
        // One after the other: 10ms + 10ms of virtual time.
        assert_eq!(start.elapsed(), Duration::from_millis(20));
    }

    #[tokio::test]
    async fn spawned_tasks_complete() {
        let results = spawn_workers(3).await;
        assert_eq!(
            results,
            ["worker-0=enabled", "worker-1=enabled", "worker-2=enabled"]
        );
    }

    #[tokio::test]
    async fn channel_delivers_in_order_and_closes() {
        assert_eq!(producer_consumer().await, [100, 200, 300, 400, 500]);
    }
}

// Exercises
// ---------
// 1. Add a `fetch_with_timeout` using tokio::time::timeout that gives
//    up after 5ms — what type does timeout return, and why?
// 2. Replace tokio::join! with futures joined one after the other and
//    measure the difference with Instant.
// 3. Swap tokio::time::sleep for std::thread::sleep in fetch_setting
//    and rerun the concurrency test. Watch it fail, understand why,
//    swap it back.
