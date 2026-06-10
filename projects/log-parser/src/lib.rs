//! Parse and summarize application logs of the form:
//!
//! ```text
//! 2026-06-02 14:31:07 [ERROR] database connection lost
//! ```
//!
//! A small, complete example of the parse → analyze pipeline:
//! FromStr for the level, a fallible line parser that *collects*
//! per-line failures instead of aborting, and iterator-driven
//! summarization.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

/// FromStr is the standard trait for "parse me from text" — it is
/// what makes `"ERROR".parse::<Level>()` work, and it composes with
/// everything else that uses parse.
impl FromStr for Level {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEBUG" => Ok(Level::Debug),
            "INFO" => Ok(Level::Info),
            "WARN" => Ok(Level::Warn),
            "ERROR" => Ok(Level::Error),
            other => Err(format!("unknown log level '{other}'")),
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
        };
        write!(f, "{name}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// Kept as text: we never compute with timestamps here, and not
    /// parsing what you don't need is a legitimate design choice.
    pub timestamp: String,
    pub level: Level,
    pub message: String,
}

/// Parse one line. Format: `<date> <time> [<LEVEL>] <message>`.
/// String errors are fine here — the caller wraps them with the line
/// number, which is the part the user actually needs.
pub fn parse_entry(line: &str) -> Result<Entry, String> {
    // split_once: split at the FIRST occurrence only — exactly right
    // for peeling fields off the front, and it returns Option so `?`
    // works via ok_or.
    let (date, rest) = line.split_once(' ').ok_or("missing timestamp")?;
    let (time, rest) = rest.split_once(' ').ok_or("missing time")?;

    let rest = rest.strip_prefix('[').ok_or("missing [LEVEL]")?;
    let (level_text, message) = rest.split_once("] ").ok_or("missing [LEVEL]")?;

    let level: Level = level_text.parse()?; // FromStr in action

    if message.is_empty() {
        return Err("empty message".to_string());
    }

    Ok(Entry {
        timestamp: format!("{date} {time}"),
        level,
        message: message.to_string(),
    })
}

/// A line that failed, with everything needed to investigate it.
#[derive(Debug, PartialEq)]
pub struct BadLine {
    pub line_number: usize,
    pub content: String,
    pub reason: String,
}

/// Parse a whole log, best-effort: real logs contain garbage
/// (truncated writes, panic spew), and "abort on first bad line" would
/// make the tool useless on exactly the files you most need to read.
/// Partition into (parsed, failures) and let the caller decide.
pub fn parse_log(text: &str) -> (Vec<Entry>, Vec<BadLine>) {
    let mut entries = Vec::new();
    let mut bad_lines = Vec::new();

    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match parse_entry(line) {
            Ok(entry) => entries.push(entry),
            Err(reason) => bad_lines.push(BadLine {
                line_number: i + 1,
                content: line.to_string(),
                reason,
            }),
        }
    }
    (entries, bad_lines)
}

/// Counts per level, for the summary report.
pub fn level_counts(entries: &[Entry]) -> HashMap<Level, usize> {
    let mut counts = HashMap::new();
    for entry in entries {
        *counts.entry(entry.level).or_insert(0) += 1;
    }
    counts
}

/// The errors, in original order — usually the first thing anyone
/// wants from a log.
pub fn errors(entries: &[Entry]) -> Vec<&Entry> {
    entries.iter().filter(|e| e.level == Level::Error).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "2026-06-02 14:31:07 [ERROR] database connection lost";

    #[test]
    fn parses_a_wellformed_line() {
        let entry = parse_entry(GOOD).unwrap();
        assert_eq!(entry.timestamp, "2026-06-02 14:31:07");
        assert_eq!(entry.level, Level::Error);
        assert_eq!(entry.message, "database connection lost");
    }

    #[test]
    fn messages_may_contain_brackets_and_spaces() {
        let entry = parse_entry("2026-06-02 09:00:00 [INFO] user [admin] logged in").unwrap();
        assert_eq!(entry.message, "user [admin] logged in");
    }

    #[test]
    fn each_failure_mode_is_described() {
        assert!(parse_entry("no-spaces-here").is_err());
        assert!(parse_entry("2026-06-02 14:31:07 no brackets").is_err());
        assert_eq!(
            parse_entry("2026-06-02 14:31:07 [TRACE] hello"),
            Err("unknown log level 'TRACE'".to_string())
        );
    }

    #[test]
    fn levels_parse_and_display_symmetrically() {
        for text in ["DEBUG", "INFO", "WARN", "ERROR"] {
            let level: Level = text.parse().unwrap();
            assert_eq!(level.to_string(), text); // round trip
        }
    }

    #[test]
    fn parse_log_is_best_effort() {
        let log = "\
2026-06-02 14:31:06 [INFO] starting up
GARBAGE LINE
2026-06-02 14:31:07 [ERROR] database connection lost

2026-06-02 14:31:09 [WARN] retrying";
        let (entries, bad) = parse_log(log);
        assert_eq!(entries.len(), 3); // good lines all survive...
        assert_eq!(bad.len(), 1); // ...and the garbage is reported,
        assert_eq!(bad[0].line_number, 2); // with its position.
    }

    #[test]
    fn summary_counts_by_level() {
        let log = "\
2026-06-02 14:31:06 [INFO] a
2026-06-02 14:31:07 [ERROR] b
2026-06-02 14:31:08 [ERROR] c";
        let (entries, _) = parse_log(log);
        let counts = level_counts(&entries);
        assert_eq!(counts.get(&Level::Error), Some(&2));
        assert_eq!(counts.get(&Level::Info), Some(&1));
        assert_eq!(counts.get(&Level::Debug), None);

        assert_eq!(errors(&entries).len(), 2);
    }
}

// Exercises
// ---------
// 1. Add `fn entries_between<'a>(entries: &'a [Entry], from: &str,
//    to: &str) -> Vec<&'a Entry>` — lexicographic comparison works on
//    these timestamps. Why? When would it break?
// 2. Find the most common ERROR message (HashMap counting again, but
//    over a filtered subset).
// 3. Make Level implement Ord-based filtering: "show WARN and above"
//    — the derive on Level already supports it; expose it.
