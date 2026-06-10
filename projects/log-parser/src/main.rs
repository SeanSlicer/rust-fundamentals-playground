//! Summarize a log file: `cargo run -p log-parser -- sample.log`

use std::process::ExitCode;

use log_parser::{errors, level_counts, parse_log, Level};

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: log-parser <file.log>");
        return ExitCode::FAILURE;
    };

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("error reading {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let (entries, bad_lines) = parse_log(&text);

    println!("== summary ==");
    let counts = level_counts(&entries);
    // Iterate the enum in severity order rather than HashMap order —
    // HashMap iteration order is deliberately unstable.
    for level in [Level::Error, Level::Warn, Level::Info, Level::Debug] {
        if let Some(count) = counts.get(&level) {
            println!("{level:>5}: {count}");
        }
    }
    println!("total: {} entries", entries.len());

    let errs = errors(&entries);
    if !errs.is_empty() {
        println!("\n== errors ==");
        for e in errs {
            println!("{} {}", e.timestamp, e.message);
        }
    }

    if !bad_lines.is_empty() {
        println!("\n== unparseable lines ==");
        for bad in &bad_lines {
            println!("line {}: {} ({})", bad.line_number, bad.content, bad.reason);
        }
    }

    ExitCode::SUCCESS
}
