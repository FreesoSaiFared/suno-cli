//! The semantic exit-code contract (0-4): deterministic triggers via the
//! hidden `contract` command, plus the natural parse and command paths.
//! Code 5 no longer exists (breaking vs 0.5.x — not-found is now 3).

mod common;
use common::{suno, suno_in};

// ── Contract command: deterministic 0-4 ────────────────────────────────────

#[test]
fn contract_exit_0() {
    suno().args(["contract", "0"]).assert().code(0);
}

#[test]
fn contract_exit_1_transient() {
    suno().args(["contract", "1"]).assert().code(1);
}

#[test]
fn contract_exit_2_config() {
    suno().args(["contract", "2"]).assert().code(2);
}

#[test]
fn contract_exit_3_bad_input() {
    suno().args(["contract", "3"]).assert().code(3);
}

#[test]
fn contract_exit_4_rate_limited() {
    suno().args(["contract", "4"]).assert().code(4);
}

#[test]
fn contract_unknown_code_is_bad_input() {
    // The removed code 5 must not resurface via the contract hook.
    suno().args(["contract", "5"]).assert().code(3);
}

// ── Natural paths ──────────────────────────────────────────────────────────

#[test]
fn help_exits_0() {
    suno().arg("--help").assert().code(0);
}

#[test]
fn version_exits_0() {
    suno().arg("--version").assert().code(0);
}

#[test]
fn missing_subcommand_exits_3() {
    suno().assert().code(3);
}

#[test]
fn unknown_flag_exits_3() {
    suno()
        .args(["list", "--definitely-not-a-flag"])
        .assert()
        .code(3);
}

// ── Required positionals: an empty ID list is bad input, not an auth call ────

#[test]
fn status_without_ids_exits_3() {
    // Missing required IDs must fail at parse (exit 3), never reach the API.
    suno().arg("status").assert().code(3);
}

#[test]
fn download_without_ids_exits_3() {
    suno().arg("download").assert().code(3);
}

#[test]
fn publish_without_ids_exits_3() {
    suno().arg("publish").assert().code(3);
}

// ── Dead flags stay dead ─────────────────────────────────────────────────────

#[test]
fn stems_wait_flag_removed_exits_3() {
    // stems accepted but ignored --wait; the flag is gone, so passing it is an
    // unknown-flag parse error rather than a silent no-op.
    suno().args(["stems", "abc", "--wait"]).assert().code(3);
}

#[test]
fn concat_wait_flag_removed_exits_3() {
    suno().args(["concat", "abc", "--wait"]).assert().code(3);
}

#[test]
fn agent_info_exits_0() {
    suno().arg("agent-info").assert().code(0);
}

#[test]
fn config_show_and_path_exit_0() {
    let tmp = tempfile::tempdir().unwrap();
    suno_in(tmp.path())
        .args(["config", "show"])
        .assert()
        .code(0);
    suno_in(tmp.path())
        .args(["config", "path"])
        .assert()
        .code(0);
}

#[test]
fn doctor_without_auth_exits_2() {
    // Fresh home: auth_file is a failing check, so doctor reports config
    // error (2) — the fix is `suno auth --login`, not a retry.
    let tmp = tempfile::tempdir().unwrap();
    let out = suno_in(tmp.path()).arg("doctor").output().unwrap();
    assert_eq!(out.status.code(), Some(2));

    // The report itself still lands on stdout so agents can read the checks.
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(report["status"], "success");
    assert!(report["data"]["checks"].is_array());
    let auth_check = report["data"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "auth_file")
        .expect("doctor must include an auth_file check");
    assert_eq!(auth_check["status"], "fail");
}
