# http-client

A GET request from first principles: `TcpStream`, four lines of
request text, and a hand-written response parser. No HTTP library.

```sh
cargo test -p http-client      # offline — parsing is pure
cargo run -p http-client       # fetches example.com (needs network, port 80)
```

## What it teaches

- **HTTP is just text over TCP.** Read `build_get_request` — that
  string is the entire mystery. `Host`, `Connection: close`, and the
  terminating blank line each get a comment explaining why they must
  be there.
- **Testable core, I/O edge.** `parse_response` is bytes-in,
  struct-out and covered by offline tests against canned responses;
  only `get` opens a socket. The tests never touch the network.
- **Collections chosen by domain** — headers live in a
  `Vec<(String, String)>`, not a HashMap, because headers repeat and
  names are case-insensitive (see `Response::header`).
- **Timeouts on every socket** — a server that never answers should
  fail your call, not hang your program.

## Limits (on purpose)

Plain HTTP only (no TLS), `Connection: close` framing only (no
Content-Length or chunked parsing), text bodies only. Each limit is a
listed exercise; HTTPS-via-rustls is the most instructive of them.
