//! An HTTP client from raw TCP sockets — no http library.
//!
//! In real code you would use `reqwest` (async) or `ureq` (blocking).
//! This crate exists to demystify them: HTTP/1.1 is just text over a
//! socket, and a GET request is four lines. Seeing that once changes
//! how you read every HTTP library's documentation afterwards.
//!
//! Design: `parse_response` is pure (bytes in, struct out) and fully
//! tested offline. Only `get` touches the network — the same
//! testable-core / thin-IO-edge split as the other projects.

use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug)]
pub enum HttpError {
    /// Connection/socket failures, wrapped from std.
    Io(std::io::Error),
    /// The server sent something that isn't valid HTTP.
    MalformedResponse(String),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpError::Io(e) => write!(f, "network error: {e}"),
            HttpError::MalformedResponse(why) => write!(f, "malformed response: {why}"),
        }
    }
}

impl std::error::Error for HttpError {}

impl From<std::io::Error> for HttpError {
    fn from(e: std::io::Error) -> Self {
        HttpError::Io(e)
    }
}

#[derive(Debug, PartialEq)]
pub struct Response {
    pub status: u16,
    pub reason: String,
    /// A Vec of pairs, not a HashMap: headers can repeat (Set-Cookie)
    /// and order is occasionally meaningful. Choosing the right
    /// collection means knowing the domain, not defaulting to maps.
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Response {
    /// Case-insensitive lookup, because header names are
    /// case-insensitive per the spec — `Content-Type` and
    /// `content-type` are the same header.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Build the request text. Minimal but correct HTTP/1.1:
/// * `Host` is mandatory in 1.1 (virtual hosting needs it).
/// * `Connection: close` tells the server to end the stream after
///   responding, so we can simply read to EOF instead of implementing
///   Content-Length / chunked framing.
/// * The blank line terminates the header block — without it the
///   server waits forever and so do you.
pub fn build_get_request(host: &str, path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
         Host: {host}\r\n\
         Connection: close\r\n\
         User-Agent: rust-fundamentals-playground/0.1\r\n\
         \r\n"
    )
}

/// Parse a raw HTTP response. Pure function: all the protocol
/// knowledge, none of the I/O, 100% testable offline.
pub fn parse_response(raw: &str) -> Result<Response, HttpError> {
    // Headers end at the first blank line; the rest is body. HTTP
    // uses \r\n line endings.
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .ok_or_else(|| HttpError::MalformedResponse("no header/body separator".into()))?;

    let mut lines = head.split("\r\n");

    // Status line: "HTTP/1.1 200 OK"
    let status_line = lines
        .next()
        .ok_or_else(|| HttpError::MalformedResponse("empty response".into()))?;
    let mut parts = status_line.splitn(3, ' ');

    let version = parts
        .next()
        .ok_or_else(|| HttpError::MalformedResponse("missing version".into()))?;
    if !version.starts_with("HTTP/") {
        return Err(HttpError::MalformedResponse(format!(
            "expected HTTP version, got '{version}'"
        )));
    }

    let status: u16 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| HttpError::MalformedResponse("bad status code".into()))?;

    // Reason phrase is optional ("HTTP/1.1 200" is legal).
    let reason = parts.next().unwrap_or("").to_string();

    // Remaining lines are "Name: value" headers.
    let mut headers = Vec::new();
    for line in lines {
        let (name, value) = line.split_once(':').ok_or_else(|| {
            HttpError::MalformedResponse(format!("header without colon: '{line}'"))
        })?;
        // Whitespace after the colon is optional per spec — trim it.
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }

    Ok(Response {
        status,
        reason,
        headers,
        body: body.to_string(),
    })
}

/// Perform a GET request. Port 80 plain TCP — no TLS, which keeps the
/// crate dependency-free; https would need rustls or native-tls (a
/// good follow-on exercise, listed below).
pub fn get(host: &str, path: &str) -> Result<Response, HttpError> {
    let mut stream = TcpStream::connect((host, 80))?;
    // Timeouts: a server that never answers should fail the call, not
    // hang the program. Easy to forget, painful to debug.
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;

    stream.write_all(build_get_request(host, path).as_bytes())?;

    // Connection: close lets us read until EOF — the server frames
    // the response by hanging up.
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;

    // Real responses may be any bytes; we keep the example to text.
    let text = String::from_utf8_lossy(&raw);
    parse_response(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Concat'd string literals keep the \r\n noise readable.
    const CANNED: &str = "HTTP/1.1 200 OK\r\n\
                          Content-Type: text/html; charset=utf-8\r\n\
                          Content-Length: 13\r\n\
                          \r\n\
                          Hello, world!";

    #[test]
    fn parses_status_headers_and_body() {
        let response = parse_response(CANNED).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.reason, "OK");
        assert_eq!(response.body, "Hello, world!");
        assert!(response.is_success());
    }

    #[test]
    fn header_lookup_is_case_insensitive() {
        let response = parse_response(CANNED).unwrap();
        assert_eq!(
            response.header("content-type"),
            Some("text/html; charset=utf-8")
        );
        assert_eq!(response.header("CONTENT-LENGTH"), Some("13"));
        assert_eq!(response.header("x-missing"), None);
    }

    #[test]
    fn error_statuses_parse_but_are_not_success() {
        let raw = "HTTP/1.1 404 Not Found\r\n\r\npage missing";
        let response = parse_response(raw).unwrap();
        assert_eq!(response.status, 404);
        assert_eq!(response.reason, "Not Found");
        assert!(!response.is_success());
    }

    #[test]
    fn reason_phrase_is_optional() {
        let response = parse_response("HTTP/1.1 204\r\n\r\n").unwrap();
        assert_eq!(response.status, 204);
        assert_eq!(response.reason, "");
    }

    #[test]
    fn malformed_responses_are_rejected() {
        assert!(matches!(
            parse_response("not http at all"),
            Err(HttpError::MalformedResponse(_))
        ));
        assert!(matches!(
            parse_response("HTTP/1.1 abc OK\r\n\r\n"),
            Err(HttpError::MalformedResponse(_))
        ));
        assert!(matches!(
            parse_response("HTTP/1.1 200 OK\r\nbroken header\r\n\r\n"),
            Err(HttpError::MalformedResponse(_))
        ));
    }

    #[test]
    fn request_builder_emits_valid_http() {
        let req = build_get_request("example.com", "/index.html");
        assert!(req.starts_with("GET /index.html HTTP/1.1\r\n"));
        assert!(req.contains("Host: example.com\r\n"));
        assert!(req.ends_with("\r\n\r\n")); // the all-important blank line
    }
}

// Exercises
// ---------
// 1. Handle redirects: on 301/302, read the Location header and retry
//    (cap the redirect count — why?).
// 2. Parse the status line with str::split_whitespace instead of
//    splitn. Which legal inputs break? (Hint: reason phrases can
//    contain spaces.)
// 3. Add HTTPS via the `rustls` crate. The parse_response function
//    should not need to change at all — verify that claim.
