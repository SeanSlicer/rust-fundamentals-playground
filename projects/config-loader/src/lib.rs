//! Layered configuration: defaults < TOML file < environment.
//!
//! The pattern used by most real services, in miniature:
//! 1. Start from compiled-in defaults (always valid).
//! 2. A config file overrides whatever keys it provides.
//! 3. Environment variables override the file (for deploys and CI).
//!
//! Serde does step 2: `#[serde(default)]` means "missing key -> keep
//! the default", so a one-line config file is legal.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
// deny_unknown_fields turns typos in the file ("prot = 80") into load
// errors instead of silently-ignored keys — kinder than it sounds.
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub verbose: bool,
    pub allowed_origins: Vec<String>,
}

/// The Default impl IS the bottom layer of the stack — every field's
/// fallback value lives here, in one reviewable place.
impl Default for Config {
    fn default() -> Self {
        Config {
            host: "127.0.0.1".to_string(),
            port: 8080,
            verbose: false,
            allowed_origins: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    /// toml's error already includes line/column context — wrap it,
    /// don't flatten it to a string.
    Parse(toml::de::Error),
    /// An env var existed but held garbage (e.g. APP_PORT=banana).
    /// Silently ignoring it would be the worst possible behavior:
    /// the operator THINKS they configured something.
    InvalidEnvVar {
        name: String,
        value: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "could not read config file: {e}"),
            ConfigError::Parse(e) => write!(f, "config file is invalid: {e}"),
            ConfigError::InvalidEnvVar { name, value } => {
                write!(f, "environment variable {name} has invalid value '{value}'")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::Parse(e)
    }
}

/// Layer 2: parse TOML over the defaults. Split from file reading so
/// it can be tested without touching the filesystem.
pub fn from_toml(text: &str) -> Result<Config, ConfigError> {
    Ok(toml::from_str(text)?)
}

/// Layers 1+2: defaults, then file (a missing file is fine — you get
/// pure defaults, same convention as the todo app).
pub fn load_file(path: &Path) -> Result<Config, ConfigError> {
    match fs::read_to_string(path) {
        Ok(text) => from_toml(&text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(e.into()),
    }
}

/// Layer 3: environment overrides. Takes a lookup closure instead of
/// calling std::env directly — tests inject a fake environment and
/// stay parallel-safe (mutating real env vars races across threads,
/// since the test harness runs tests concurrently).
pub fn apply_env_overrides(
    mut config: Config,
    get_var: impl Fn(&str) -> Option<String>,
) -> Result<Config, ConfigError> {
    if let Some(host) = get_var("APP_HOST") {
        config.host = host;
    }
    if let Some(port) = get_var("APP_PORT") {
        config.port = port.parse().map_err(|_| ConfigError::InvalidEnvVar {
            name: "APP_PORT".to_string(),
            value: port,
        })?;
    }
    if let Some(verbose) = get_var("APP_VERBOSE") {
        config.verbose = match verbose.as_str() {
            "1" | "true" | "yes" => true,
            "0" | "false" | "no" => false,
            other => {
                return Err(ConfigError::InvalidEnvVar {
                    name: "APP_VERBOSE".to_string(),
                    value: other.to_string(),
                })
            }
        };
    }
    Ok(config)
}

/// The full stack, as the application would call it.
pub fn load(path: &Path) -> Result<Config, ConfigError> {
    let config = load_file(path)?;
    apply_env_overrides(config, |name| std::env::var(name).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_means_all_defaults() {
        assert_eq!(from_toml("").unwrap(), Config::default());
    }

    #[test]
    fn partial_file_overrides_only_named_keys() {
        let config = from_toml("port = 9000").unwrap();
        assert_eq!(config.port, 9000);
        assert_eq!(config.host, "127.0.0.1"); // untouched default
    }

    #[test]
    fn full_file_parses() {
        let config = from_toml(
            r#"
                host = "0.0.0.0"
                port = 443
                verbose = true
                allowed_origins = ["https://example.com"]
            "#,
        )
        .unwrap();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.allowed_origins, ["https://example.com"]);
    }

    #[test]
    fn typos_are_rejected_not_ignored() {
        // deny_unknown_fields at work: "prot" is not a field.
        assert!(matches!(from_toml("prot = 80"), Err(ConfigError::Parse(_))));
    }

    #[test]
    fn type_errors_are_parse_errors() {
        assert!(from_toml("port = \"eighty\"").is_err());
        assert!(from_toml("port = 99999").is_err()); // doesn't fit u16
    }

    #[test]
    fn missing_file_gives_defaults() {
        let path = std::env::temp_dir().join("config_loader_test_missing.toml");
        let _ = std::fs::remove_file(&path);
        assert_eq!(load_file(&path).unwrap(), Config::default());
    }

    #[test]
    fn env_overrides_beat_file_values() {
        let from_file = from_toml("port = 9000").unwrap();
        // Fake environment: no real env vars touched, test stays
        // parallel-safe.
        let config = apply_env_overrides(from_file, |name| match name {
            "APP_PORT" => Some("9001".to_string()),
            "APP_VERBOSE" => Some("yes".to_string()),
            _ => None,
        })
        .unwrap();
        assert_eq!(config.port, 9001); // env wins over file
        assert!(config.verbose);
        assert_eq!(config.host, "127.0.0.1"); // untouched layers shine through
    }

    #[test]
    fn garbage_env_values_error_loudly() {
        let result = apply_env_overrides(Config::default(), |name| {
            (name == "APP_PORT").then(|| "banana".to_string())
        });
        assert!(matches!(
            result,
            Err(ConfigError::InvalidEnvVar { ref name, .. }) if name == "APP_PORT"
        ));
    }
}

// Exercises
// ---------
// 1. Add a `timeout_secs: u64` field. How many places need to change?
//    (Counting them tells you how well-factored the layering is.)
// 2. Add an APP_ALLOWED_ORIGINS override using a comma-separated
//    value. Decide: replace the list, or append to it?
// 3. Add a `validate()` step after loading (e.g. port != 0, host
//    non-empty). Should validation errors be a new ConfigError
//    variant or a separate error type?
