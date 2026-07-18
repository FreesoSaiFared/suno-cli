//! Envelope discipline: success envelopes on stdout, error envelopes on
//! stderr (schema-valid), help/version wrapped when piped. assert_cmd runs
//! the binary in a pipe (not a TTY), so JSON auto-detection is active.

mod common;
use common::{suno, vendored_schema};

fn assert_valid_error_envelope(stderr: &[u8]) -> serde_json::Value {
    let json: serde_json::Value =
        serde_json::from_slice(stderr).expect("stderr should carry a JSON error envelope");
    let schema = vendored_schema("envelope.schema.json");
    if let Err(e) = jsonschema::validate(&schema, &json) {
        panic!("error envelope violates the vendored framework schema: {e}");
    }
    json
}

// ── Success envelope ───────────────────────────────────────────────────────

#[test]
fn success_envelope_shape() {
    let out = suno().args(["contract", "0"]).output().unwrap();
    assert!(out.status.success());

    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["version"], "1");
    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["contract"], true);
}

#[test]
fn quiet_still_emits_json() {
    // --quiet suppresses stderr chatter, never the envelope.
    let out = suno()
        .args(["contract", "0", "--json", "--quiet"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["status"], "success");
}

// ── Error envelope ─────────────────────────────────────────────────────────

#[test]
fn unknown_command_is_a_clean_json_error() {
    let out = suno().arg("definitely-not-a-command-xyz").output().unwrap();
    assert_eq!(out.status.code(), Some(3));
    assert!(out.stdout.is_empty(), "errors must not pollute stdout");

    let json = assert_valid_error_envelope(&out.stderr);
    assert_eq!(json["error"]["code"], "invalid_input");
}

#[test]
fn parse_error_wrapped_in_envelope() {
    // `describe` requires --prompt.
    let out = suno().arg("describe").output().unwrap();
    assert_eq!(out.status.code(), Some(3));

    let json = assert_valid_error_envelope(&out.stderr);
    assert_eq!(json["error"]["code"], "invalid_input");
    assert!(
        json["error"]["suggestion"]
            .as_str()
            .unwrap()
            .contains("--help"),
        "parse errors must point at --help"
    );
}

#[test]
fn error_code_matches_exit_code_semantics() {
    let cases = [
        ("1", "transient"),
        ("2", "config_error"),
        ("4", "rate_limited"),
    ];
    for (code, expected) in cases {
        let out = suno().args(["contract", code]).output().unwrap();
        let json = assert_valid_error_envelope(&out.stderr);
        assert_eq!(json["error"]["code"], expected, "contract {code}");
    }
}

// ── Help/version wrapping ──────────────────────────────────────────────────

#[test]
fn piped_help_wrapped_with_tips_and_examples() {
    let out = suno().arg("--help").output().unwrap();
    assert!(out.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("piped --help should be a JSON envelope");
    assert_eq!(json["status"], "success");

    // Pattern 6: agents bootstrap usage from --help alone.
    let usage = json["data"]["usage"].as_str().unwrap();
    assert!(
        usage.contains("Tips:"),
        "--help must include a Tips section"
    );
    assert!(
        usage.contains("Examples:"),
        "--help must include an Examples section"
    );
}

#[test]
fn piped_version_wrapped_in_envelope() {
    let out = suno().arg("--version").output().unwrap();
    assert!(out.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("piped --version should be a JSON envelope");
    assert_eq!(json["status"], "success");
}
