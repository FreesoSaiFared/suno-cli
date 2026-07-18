//! Configuration with 3-tier precedence: compiled defaults < TOML file
//! (`config path` shows where) < `SUNO_*` env vars. Command-line flags beat
//! all three at the call sites that consume these values.

use std::path::PathBuf;

use figment::{
    Figment,
    providers::{Env, Format as _, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

use crate::errors::CliError;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// Default --model for generate/describe/extend/cover (clap name,
    /// e.g. "v5.5" — not the chirp-* API key).
    pub default_model: String,
    /// Initial poll backoff for --wait (doubles up to 15s).
    pub poll_interval_secs: u64,
    /// Total --wait timeout before giving up on a generation.
    pub poll_timeout_secs: u64,
    /// Default directory for `download` when -o is not given.
    pub output_dir: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_model: "v5.5".into(),
            poll_interval_secs: 5,
            poll_timeout_secs: 600,
            output_dir: ".".into(),
        }
    }
}

pub fn config_path() -> PathBuf {
    directories::ProjectDirs::from("com", "suno-cli", "suno-cli")
        .map(|d| d.config_dir().join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("~/.config/suno-cli/config.toml"))
}

/// State directory (duplicate-guard locks, solver Chrome profile).
pub fn data_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "suno-cli", "suno-cli")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub const CONFIG_KEYS: &[&str] = &[
    "default_model",
    "poll_interval_secs",
    "poll_timeout_secs",
    "output_dir",
];

impl AppConfig {
    /// Env keys are flat (`SUNO_POLL_INTERVAL_SECS` → `poll_interval_secs`),
    /// so no `.split("_")` — splitting would shred multi-word field names
    /// into nonexistent nested tables.
    pub fn load() -> Result<Self, CliError> {
        Figment::from(Serialized::defaults(AppConfig::default()))
            .merge(Toml::file(config_path()))
            .merge(Env::prefixed("SUNO_"))
            .extract()
            .map_err(|e| CliError::Config(format!("config: {e}")))
    }

    /// Validate and persist one key to the TOML file, preserving any keys
    /// already there (including ones this version doesn't know about).
    pub fn set_value(key: &str, value: &str) -> Result<PathBuf, CliError> {
        let parsed = match key {
            "poll_interval_secs" | "poll_timeout_secs" => {
                let n: u64 = value.parse().map_err(|_| {
                    CliError::InvalidInput(format!("{key} must be a positive integer, got {value}"))
                })?;
                toml::Value::Integer(n as i64)
            }
            "default_model" => {
                <crate::cli::ModelVersion as clap::ValueEnum>::from_str(value, true).map_err(
                    |_| {
                        CliError::InvalidInput(format!(
                            "unknown model '{value}' — see `suno generate --help` for valid --model values"
                        ))
                    },
                )?;
                toml::Value::String(value.into())
            }
            "output_dir" => toml::Value::String(value.into()),
            other => {
                return Err(CliError::InvalidInput(format!(
                    "unknown config key '{other}' — valid keys: {}",
                    CONFIG_KEYS.join(", ")
                )));
            }
        };

        let path = config_path();
        let mut table: toml::Table = if path.exists() {
            toml::from_str(&std::fs::read_to_string(&path)?)
                .map_err(|e| CliError::Config(format!("cannot parse {}: {e}", path.display())))?
        } else {
            toml::Table::new()
        };
        table.insert(key.into(), parsed);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let serialized = toml::to_string_pretty(&table)
            .map_err(|e| CliError::Config(format!("config serialize: {e}")))?;
        std::fs::write(&path, serialized)?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // figment::Jail's closure returns figment's own large Error type;
    // nothing we can shrink on our side.
    #[allow(clippy::result_large_err)]
    fn env_overrides_defaults_without_splitting() {
        // Flat env keys must map onto multi-word field names; a `.split("_")`
        // provider would break every key in this config.
        figment::Jail::expect_with(|jail| {
            jail.set_env("SUNO_POLL_INTERVAL_SECS", "99");
            jail.set_env("SUNO_DEFAULT_MODEL", "v4.5");
            let cfg = AppConfig::load().expect("load");
            assert_eq!(cfg.poll_interval_secs, 99);
            assert_eq!(cfg.default_model, "v4.5");
            // Untouched keys keep their defaults.
            assert_eq!(cfg.poll_timeout_secs, 600);
            Ok(())
        });
    }

    #[test]
    fn set_value_rejects_unknown_keys_and_bad_values() {
        assert!(matches!(
            AppConfig::set_value("nope", "1"),
            Err(CliError::InvalidInput(_))
        ));
        assert!(matches!(
            AppConfig::set_value("poll_interval_secs", "fast"),
            Err(CliError::InvalidInput(_))
        ));
        assert!(matches!(
            AppConfig::set_value("default_model", "v99"),
            Err(CliError::InvalidInput(_))
        ));
    }
}
