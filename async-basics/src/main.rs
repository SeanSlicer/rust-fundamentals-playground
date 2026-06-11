//! A runnable tour of the library: `cargo run -p async-basics`.
//!
//! `main` cannot be `async` on its own — someone must own the
//! runtime. #[tokio::main] expands to a normal main that builds a
//! Tokio runtime and calls block_on(async { ... }).

use std::time::Instant;

use async_basics::{fetch_two_concurrent, fetch_two_sequential, producer_consumer, spawn_workers};

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let (a, b) = fetch_two_sequential().await;
    println!("sequential: {a}, {b}  ({:?})", start.elapsed());

    let start = Instant::now();
    let (a, b) = fetch_two_concurrent().await;
    println!("concurrent: {a}, {b}  ({:?})", start.elapsed());

    let workers = spawn_workers(3).await;
    println!("spawned tasks: {workers:?}");

    let received = producer_consumer().await;
    println!("channel received: {received:?}");
}
