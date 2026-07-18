//! Distribution-aware update: a package-manager-owned binary must NEVER be
//! self-replaced (overwriting a brew Cellar symlink target corrupts the
//! install). SUNO_INSTALL_SOURCE overrides exe-path detection so these run
//! offline against any build location.

mod common;
use common::{skip_live, suno};

fn update_json(source: &str, extra: &[&str]) -> (Option<i32>, Vec<u8>, Vec<u8>) {
    let mut args = vec!["update"];
    args.extend_from_slice(extra);
    let out = suno()
        .env("SUNO_INSTALL_SOURCE", source)
        .args(&args)
        .output()
        .unwrap();
    (out.status.code(), out.stdout, out.stderr)
}

#[test]
fn homebrew_full_update_never_self_replaces() {
    let bin = assert_cmd::cargo::cargo_bin("suno");
    let before = std::fs::metadata(&bin).unwrap().modified().unwrap();

    // Full `update` (not --check): the managed path must return the owner
    // channel's command without touching the network or the binary.
    let (code, stdout, _) = update_json("homebrew", &[]);
    assert_eq!(code, Some(0));

    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(json["status"], "success");
    assert_eq!(json["data"]["status"], "managed_install");
    assert_eq!(json["data"]["install_source"], "homebrew");
    assert_eq!(json["data"]["update_mode"], "package_manager");
    assert_eq!(
        json["data"]["upgrade_command"],
        "brew upgrade paperfoot/tap/suno"
    );

    let after = std::fs::metadata(&bin).unwrap().modified().unwrap();
    assert_eq!(before, after, "update must not touch a brew-owned binary");
}

#[test]
fn homebrew_check_is_also_managed() {
    let (code, stdout, _) = update_json("homebrew", &["--check"]);
    assert_eq!(code, Some(0));
    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(json["data"]["status"], "managed_install");
}

#[test]
fn cargo_update_returns_cargo_install_command() {
    let (code, stdout, _) = update_json("cargo", &[]);
    assert_eq!(code, Some(0));
    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(json["data"]["install_source"], "cargo");
    assert_eq!(json["data"]["update_mode"], "package_manager");
    assert_eq!(
        json["data"]["upgrade_command"],
        "cargo install --locked --force suno"
    );
}

#[test]
fn brew_is_accepted_as_homebrew_alias() {
    let (code, stdout, _) = update_json("brew", &["--check"]);
    assert_eq!(code, Some(0));
    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(json["data"]["install_source"], "homebrew");
}

#[test]
fn invalid_install_source_exits_2() {
    let (code, stdout, stderr) = update_json("spaceship", &["--check"]);
    assert_eq!(code, Some(2));
    assert!(stdout.is_empty());

    let json: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "config_error");
}

#[test]
fn unknown_source_fails_closed() {
    // Provenance we can't classify must never self-replace: exit 2 with
    // reinstall guidance, and the binary on disk left untouched.
    let bin = assert_cmd::cargo::cargo_bin("suno");
    let before = std::fs::metadata(&bin).unwrap().modified().unwrap();

    let (code, stdout, stderr) = update_json("unknown", &[]);
    assert_eq!(code, Some(2));
    assert!(stdout.is_empty());
    let json: serde_json::Value = serde_json::from_slice(&stderr).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "config_error");

    let after = std::fs::metadata(&bin).unwrap().modified().unwrap();
    assert_eq!(
        before, after,
        "fail-closed update must not touch the binary"
    );
}

#[test]
fn standalone_check_reaches_github_live() {
    // Network-touching: standalone --check queries GitHub Releases.
    if skip_live() {
        return;
    }
    let (code, stdout, _) = update_json("standalone", &["--check"]);
    assert_eq!(code, Some(0));
    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    let status = json["data"]["status"].as_str().unwrap();
    assert!(
        status == "up_to_date" || status == "update_available",
        "unexpected status: {status}"
    );
}
