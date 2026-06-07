//! Show the effective configuration after all layers are applied.
//!
//! Try it:
//! ```sh
//! cargo run -p config-loader                       # pure defaults
//! cargo run -p config-loader -- app.example.toml   # defaults + file
//! # then set APP_PORT=9999 in your shell and run again
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("app.toml"));

    match config_loader::load(&path) {
        Ok(config) => {
            println!("effective configuration (from {}):", path.display());
            println!("{config:#?}"); // pretty Debug — fine for an admin tool
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
