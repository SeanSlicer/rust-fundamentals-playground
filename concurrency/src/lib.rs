//! Threads, channels, and shared state.
//!
//! Rust's slogan "fearless concurrency" is concrete: the Send and
//! Sync marker traits let the compiler reject data races at compile
//! time. The ownership errors you learned to fix in single-threaded
//! code are the SAME errors that prevent races here.

pub mod threads;
