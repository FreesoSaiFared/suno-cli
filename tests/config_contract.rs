//! The config layer must be real, not decorative: file values load, SUNO_*
//! env vars override them, and `config set` persists. Regression guard for
//! the 0.5.x bug where the config file existed but nothing read it.

mod common;
use common::{config_path_in, suno_in, write_config_in};

fn show_json(cmd: &mut assert_cmd::Command) -> serde_json::Value {
    let out = cmd.args(["config", "show"]).output().unwrap();
    assert!(out.status.success());
    serde_json::from_slice(&out.stdout).expect("config show should be JSON")
}

#[test]
fn defaults_apply_in_empty_home() {
    let tmp = tempfile::tempdir().unwrap();
    let json = show_json(&mut suno_in(tmp.path()));
    assert_eq!(json["data"]["default_model"], "v5.5");
    assert_eq!(json["data"]["poll_interval_secs"], 5);
    assert_eq!(json["data"]["poll_timeout_secs"], 600);
}

#[test]
fn env_var_overrides_default() {
    let tmp = tempfile::tempdir().unwrap();
    let json = show_json(suno_in(tmp.path()).env("SUNO_POLL_INTERVAL_SECS", "99"));
    assert_eq!(json["data"]["poll_interval_secs"], 99);
    // Untouched keys keep their defaults.
    assert_eq!(json["data"]["poll_timeout_secs"], 600);
}

#[test]
fn config_file_overrides_default() {
    let tmp = tempfile::tempdir().unwrap();
    write_config_in(tmp.path(), "poll_interval_secs = 42\n");
    let json = show_json(&mut suno_in(tmp.path()));
    assert_eq!(json["data"]["poll_interval_secs"], 42);
}

#[test]
fn env_var_beats_config_file() {
    let tmp = tempfile::tempdir().unwrap();
    write_config_in(tmp.path(), "poll_interval_secs = 42\n");
    let json = show_json(suno_in(tmp.path()).env("SUNO_POLL_INTERVAL_SECS", "99"));
    assert_eq!(json["data"]["poll_interval_secs"], 99);
}

#[test]
fn config_set_persists_to_file() {
    let tmp = tempfile::tempdir().unwrap();
    suno_in(tmp.path())
        .args(["config", "set", "poll_interval_secs", "7"])
        .assert()
        .code(0);

    let path = config_path_in(tmp.path());
    let contents = std::fs::read_to_string(path).unwrap();
    assert!(contents.contains("poll_interval_secs = 7"));

    let json = show_json(&mut suno_in(tmp.path()));
    assert_eq!(json["data"]["poll_interval_secs"], 7);
}

#[test]
fn config_set_rejects_unknown_key() {
    let tmp = tempfile::tempdir().unwrap();
    suno_in(tmp.path())
        .args(["config", "set", "nope", "1"])
        .assert()
        .code(3);
}

#[test]
fn config_set_rejects_invalid_model() {
    let tmp = tempfile::tempdir().unwrap();
    suno_in(tmp.path())
        .args(["config", "set", "default_model", "v99"])
        .assert()
        .code(3);
}

#[test]
fn config_check_passes_on_clean_home() {
    let tmp = tempfile::tempdir().unwrap();
    suno_in(tmp.path())
        .args(["config", "check"])
        .assert()
        .code(0);
}

#[test]
fn config_check_fails_on_malformed_file() {
    let tmp = tempfile::tempdir().unwrap();
    write_config_in(tmp.path(), "poll_interval_secs = \"not a number\"\n");
    let out = suno_in(tmp.path())
        .args(["config", "check"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));

    let json: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(json["error"]["code"], "config_error");
}
