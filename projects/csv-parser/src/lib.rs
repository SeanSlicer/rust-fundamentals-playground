//! A CSV parser written by hand, state machine and all.
//!
//! In production you would use the `csv` crate — this exists to learn
//! what such a crate does for you. CSV looks trivial ("split on
//! commas") until quoting arrives:
//!
//! ```text
//! name,note
//! widget,"3,5 mm, ""coarse"" thread"
//! ```
//!
//! That second field contains commas and quotes, so a real parser
//! must track whether it is inside quotes — a textbook small state
//! machine. (We parse line-by-line for clarity; embedded newlines
//! inside quoted fields are left as the big exercise.)

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum CsvError {
    /// A quote appeared mid-field, e.g. `ab"cd` — per RFC 4180 quotes
    /// must wrap the whole field.
    StrayQuote { line: usize, column: usize },
    /// The line ended while a quoted field was still open.
    UnterminatedQuote { line: usize },
    /// A data row's field count doesn't match the header's.
    WrongFieldCount {
        line: usize,
        expected: usize,
        found: usize,
    },
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::StrayQuote { line, column } => {
                write!(f, "line {line}: unexpected quote at column {column}")
            }
            CsvError::UnterminatedQuote { line } => {
                write!(f, "line {line}: quoted field never closed")
            }
            CsvError::WrongFieldCount {
                line,
                expected,
                found,
            } => write!(f, "line {line}: expected {expected} fields, found {found}"),
        }
    }
}

impl std::error::Error for CsvError {}

/// The state machine states. An enum (not two booleans) so impossible
/// combinations cannot be represented.
#[derive(Debug, PartialEq, Clone, Copy)]
enum State {
    /// At the start of a field — quoting is still allowed.
    FieldStart,
    /// Inside an unquoted field.
    Unquoted,
    /// Inside a quoted field.
    Quoted,
    /// Just saw a quote while Quoted: either the field ends, or it is
    /// the first half of an escaped quote ("").
    QuoteInQuoted,
}

/// Parse a single CSV line into fields. `line_number` is only for
/// error messages — threading position info through a parser is what
/// makes its errors actually helpful.
pub fn parse_line(line: &str, line_number: usize) -> Result<Vec<String>, CsvError> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut state = State::FieldStart;

    for (column, c) in line.chars().enumerate() {
        // The entire format, one (state, char) table. Every arm is a
        // transition; match exhaustiveness proves none is forgotten.
        state = match (state, c) {
            (State::FieldStart, '"') => State::Quoted,
            (State::FieldStart, ',') => {
                fields.push(std::mem::take(&mut current)); // empty field
                State::FieldStart
            }
            (State::FieldStart, c) => {
                current.push(c);
                State::Unquoted
            }

            (State::Unquoted, ',') => {
                // take() moves the String out and leaves a fresh empty
                // one — no clone, no manual reset.
                fields.push(std::mem::take(&mut current));
                State::FieldStart
            }
            (State::Unquoted, '"') => {
                return Err(CsvError::StrayQuote {
                    line: line_number,
                    column: column + 1,
                })
            }
            (State::Unquoted, c) => {
                current.push(c);
                State::Unquoted
            }

            (State::Quoted, '"') => State::QuoteInQuoted,
            (State::Quoted, c) => {
                current.push(c);
                State::Quoted
            }

            // "" inside quotes is an escaped quote character.
            (State::QuoteInQuoted, '"') => {
                current.push('"');
                State::Quoted
            }
            (State::QuoteInQuoted, ',') => {
                fields.push(std::mem::take(&mut current));
                State::FieldStart
            }
            (State::QuoteInQuoted, _) => {
                return Err(CsvError::StrayQuote {
                    line: line_number,
                    column: column + 1,
                })
            }
        };
    }

    // End of line: which states are legal stopping points?
    match state {
        State::Quoted => Err(CsvError::UnterminatedQuote { line: line_number }),
        _ => {
            fields.push(current);
            Ok(fields)
        }
    }
}

/// A parsed document: header plus rows, with field counts validated.
#[derive(Debug, PartialEq)]
pub struct Document {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Document {
    /// Column lookup by header name — the ergonomic accessor that
    /// makes the struct worth having over Vec<Vec<String>>.
    pub fn column(&self, name: &str) -> Option<Vec<&str>> {
        let index = self.headers.iter().position(|h| h == name)?;
        Some(self.rows.iter().map(|row| row[index].as_str()).collect())
    }
}

/// Parse a whole document. Blank lines are skipped (common in
/// hand-edited files); every row must match the header width.
pub fn parse(text: &str) -> Result<Document, CsvError> {
    let mut lines = text
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty());

    let headers = match lines.next() {
        Some((i, line)) => parse_line(line, i + 1)?,
        None => {
            return Ok(Document {
                headers: Vec::new(),
                rows: Vec::new(),
            })
        }
    };

    let mut rows = Vec::new();
    for (i, line) in lines {
        let row = parse_line(line, i + 1)?;
        if row.len() != headers.len() {
            return Err(CsvError::WrongFieldCount {
                line: i + 1,
                expected: headers.len(),
                found: row.len(),
            });
        }
        rows.push(row);
    }

    Ok(Document { headers, rows })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_fields_split_on_commas() {
        assert_eq!(
            parse_line("a,b,c", 1),
            Ok(vec!["a".into(), "b".into(), "c".into()])
        );
    }

    #[test]
    fn empty_fields_are_preserved() {
        assert_eq!(
            parse_line("a,,c", 1),
            Ok(vec!["a".into(), "".into(), "c".into()])
        );
        assert_eq!(parse_line(",", 1), Ok(vec!["".into(), "".into()]));
        // One empty line = one empty field; CSV has no zero-field rows.
        assert_eq!(parse_line("", 1), Ok(vec!["".into()]));
    }

    #[test]
    fn quoted_fields_may_contain_commas() {
        assert_eq!(
            parse_line(r#"widget,"3,5 mm",cheap"#, 1),
            Ok(vec!["widget".into(), "3,5 mm".into(), "cheap".into()])
        );
    }

    #[test]
    fn doubled_quotes_escape_quotes() {
        assert_eq!(
            parse_line(r#""she said ""hi""",x"#, 1),
            Ok(vec![r#"she said "hi""#.into(), "x".into()])
        );
    }

    #[test]
    fn quote_errors_carry_positions() {
        assert_eq!(
            parse_line(r#"ab"cd"#, 7),
            Err(CsvError::StrayQuote { line: 7, column: 3 })
        );
        assert_eq!(
            parse_line(r#""never closed"#, 3),
            Err(CsvError::UnterminatedQuote { line: 3 })
        );
    }

    #[test]
    fn document_parsing_validates_width() {
        let doc = parse("name,age\nana,31\nbob,28\n").unwrap();
        assert_eq!(doc.headers, ["name", "age"]);
        assert_eq!(doc.rows.len(), 2);

        let err = parse("name,age\nana\n").unwrap_err();
        assert_eq!(
            err,
            CsvError::WrongFieldCount {
                line: 2,
                expected: 2,
                found: 1
            }
        );
    }

    #[test]
    fn column_extraction() {
        let doc = parse("name,age\nana,31\nbob,28\n").unwrap();
        assert_eq!(doc.column("age"), Some(vec!["31", "28"]));
        assert_eq!(doc.column("height"), None);
    }

    #[test]
    fn blank_lines_are_skipped() {
        let doc = parse("name\n\nana\n\n").unwrap();
        assert_eq!(doc.rows.len(), 1);
    }
}

// Exercises
// ---------
// 1. The big one: support newlines inside quoted fields. parse_line's
//    signature can't survive — design the replacement (hint: iterate
//    chars of the whole document, or return "need more input").
// 2. Add an iterator API: `fn records(text: &str) -> impl Iterator
//    <Item = Result<Vec<String>, CsvError>> + '_` that parses lazily.
// 3. Make the delimiter configurable (';' is common in Europe) without
//    five copies of the state machine.
