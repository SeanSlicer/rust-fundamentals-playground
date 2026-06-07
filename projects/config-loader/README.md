# config-loader

Layered configuration the way real services do it:

```text
compiled-in defaults  <  TOML file  <  environment variables
```

```sh
cargo run -p config-loader -- app.example.toml
cargo test -p config-loader
```

Then override a layer from your shell and watch it win:

```powershell
$env:APP_PORT = "9999"; cargo run -p config-loader -- app.example.toml
```

## What it teaches

- **`#[serde(default)]`** — partial config files are legal; missing
  keys fall through to `Config::default()`.
- **`deny_unknown_fields`** — typos in the file become load errors
  instead of silently-ignored keys. Debugging "why is my setting not
  applied" is miserable; this kills the whole category.
- **Dependency injection for testability** — `apply_env_overrides`
  takes a lookup *closure* rather than reading `std::env` directly,
  so tests inject a fake environment and stay parallel-safe. This is
  the single most reusable trick in the crate.
- **Errors that name the layer** — file unreadable, file invalid, and
  env var garbage are three different operator mistakes; the error
  enum keeps them distinguishable.
