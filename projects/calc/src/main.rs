//! CLI driver: evaluate arguments, or run a tiny REPL when given none.
//!
//! All the interesting logic lives in lib.rs where it is testable —
//! main.rs only does I/O. Keeping the binary this thin is the
//! standard layout for Rust CLI tools.

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        repl();
        return ExitCode::SUCCESS;
    }

    // Joining args lets the shell split however it likes:
    // `calc 1 + 2` and `calc "1 + 2"` both work.
    let expression = args.join(" ");
    match calc::evaluate(&expression) {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            // Errors go to stderr so stdout stays pipeable.
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn repl() {
    println!("calc — enter an expression, or 'quit' to exit");
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().expect("stdout flush");

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            break; // EOF (ctrl-z / ctrl-d)
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "quit" || line == "exit" {
            break;
        }
        match calc::evaluate(line) {
            Ok(value) => println!("{value}"),
            Err(e) => eprintln!("error: {e}"),
        }
    }
}
