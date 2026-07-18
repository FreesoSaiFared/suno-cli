//! Shared test helpers: run the suno binary against a fully isolated home.
//!
//! Overriding HOME alone is NOT enough. GitHub's Linux runners export
//! XDG_CONFIG_HOME, which the `directories` crate prefers over HOME, so
//! tests that only set HOME silently read and write the runner's real
//! config directory — polluting every other test. Point HOME and all XDG
//! dirs at the temp home so the child process is hermetic on every
//! platform (agent-cli-framework commit a7797eb).

#![allow(dead_code)] // not every test file uses every helper

use assert_cmd::Command;
use std::path::{Path, PathBuf};

/// SUNO_* vars that would leak the developer's real environment into a test
/// run (config overrides, install-source override, Chrome override).
const AMBIENT_ENV: &[&str] = &[
    "SUNO_DEFAULT_MODEL",
    "SUNO_POLL_INTERVAL_SECS",
    "SUNO_POLL_TIMEOUT_SECS",
    "SUNO_OUTPUT_DIR",
    "SUNO_INSTALL_SOURCE",
    "SUNO_CHROME_PATH",
    "SUNO_LIVE_TESTS",
];

/// The suno binary with ambient SUNO_* scrubbed. For commands that touch
/// neither config nor state (contract, --help); everything else should use
/// `suno_in` with a temp home.
pub fn suno() -> Command {
    let mut cmd = Command::cargo_bin("suno").unwrap();
    for key in AMBIENT_ENV {
        cmd.env_remove(key);
    }
    cmd
}

/// The suno binary with HOME and XDG dirs isolated to `home`.
pub fn suno_in(home: &Path) -> Command {
    let mut cmd = suno();
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", home.join(".local/share"))
        .env("XDG_CACHE_HOME", home.join(".cache"));
    cmd
}

/// Ask the binary where its config file lives for this home. Discovery via
/// `config path` keeps tests platform-correct (macOS uses
/// Library/Application Support, Linux uses ~/.config) instead of hardcoding
/// one platform's layout.
pub fn config_path_in(home: &Path) -> PathBuf {
    let out = suno_in(home)
        .args(["config", "path", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("config path should be JSON");
    PathBuf::from(json["data"]["path"].as_str().unwrap())
}

/// Write a config file into an isolated home, creating parent dirs.
pub fn write_config_in(home: &Path, contents: &str) -> PathBuf {
    let path = config_path_in(home);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, contents).unwrap();
    path
}

/// Parse the agent-info manifest (raw JSON, not enveloped).
pub fn agent_info() -> serde_json::Value {
    let out = suno().arg("agent-info").output().unwrap();
    assert!(out.status.success());
    serde_json::from_slice(&out.stdout).expect("agent-info must be valid JSON")
}

/// Load a vendored framework schema from conformance/schemas/.
pub fn vendored_schema(name: &str) -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("conformance/schemas")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap())
        .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", path.display()))
}

/// True unless SUNO_LIVE_TESTS=1 — network-touching tests bail out early so
/// CI stays hermetic.
pub fn skip_live() -> bool {
    if std::env::var("SUNO_LIVE_TESTS").as_deref() == Ok("1") {
        false
    } else {
        eprintln!("skipped: set SUNO_LIVE_TESTS=1 to run network-touching tests");
        true
    }
}
