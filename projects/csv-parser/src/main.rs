//! Pretty-print a CSV file: `cargo run -p csv-parser -- people.csv`.

use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: csv-parser <file.csv>");
        return ExitCode::FAILURE;
    };

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("error reading {path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    match csv_parser::parse(&text) {
        Ok(doc) => {
            // Width per column = widest cell, so the table lines up.
            let widths: Vec<usize> = doc
                .headers
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    doc.rows
                        .iter()
                        .map(|row| row[i].len())
                        .chain(std::iter::once(h.len()))
                        .max()
                        .unwrap_or(0)
                })
                .collect();

            print_row(&doc.headers, &widths);
            for row in &doc.rows {
                print_row(row, &widths);
            }
            println!("({} rows)", doc.rows.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            // The parser's position-aware errors pay off here: the
            // user gets a line and column, not just "bad file".
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn print_row(cells: &[String], widths: &[usize]) {
    let line: Vec<String> = cells
        .iter()
        .zip(widths)
        .map(|(cell, width)| format!("{cell:width$}"))
        .collect();
    println!("{}", line.join(" | "));
}
