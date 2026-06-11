//! Fetch a page over plain HTTP and print the result.
//!
//! Needs network access (port 80 outbound):
//! ```sh
//! cargo run -p http-client                      # fetches example.com
//! cargo run -p http-client -- example.com /
//! ```
//! The protocol logic itself is tested offline — see lib.rs.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (host, path) = match args.as_slice() {
        [] => ("example.com".to_string(), "/".to_string()),
        [host] => (host.clone(), "/".to_string()),
        [host, path, ..] => (host.clone(), path.clone()),
    };

    println!("GET http://{host}{path}\n");

    match http_client::get(&host, &path) {
        Ok(response) => {
            println!("status: {} {}", response.status, response.reason);
            if let Some(content_type) = response.header("content-type") {
                println!("content-type: {content_type}");
            }
            println!("body: {} bytes", response.body.len());

            // Show the first few lines, not a screenful of HTML.
            for line in response.body.lines().take(8) {
                println!("| {line}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
