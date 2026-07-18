//! Skill management: install writes to every detected agent platform under
//! the (isolated) home, re-install is idempotent, and the 0.5.x
//! `install-skill` spelling still routes.

mod common;
use common::suno_in;

#[test]
fn install_writes_all_platforms() {
    let tmp = tempfile::tempdir().unwrap();
    let out = suno_in(tmp.path())
        .args(["skill", "install"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));

    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["status"], "success");

    let entries = json["data"].as_array().unwrap();
    assert!(!entries.is_empty());
    for entry in entries {
        assert_eq!(entry["status"], "installed");
        let path = entry["path"].as_str().unwrap();
        assert!(
            std::path::Path::new(path).is_file(),
            "skill file missing: {path}"
        );
        assert!(path.starts_with(tmp.path().to_str().unwrap()));
    }
}

#[test]
fn reinstall_is_idempotent() {
    let tmp = tempfile::tempdir().unwrap();
    suno_in(tmp.path())
        .args(["skill", "install"])
        .assert()
        .code(0);

    let out = suno_in(tmp.path())
        .args(["skill", "install"])
        .output()
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    for entry in json["data"].as_array().unwrap() {
        assert_eq!(entry["status"], "already_current");
    }
}

#[test]
fn status_reports_installed_and_current() {
    let tmp = tempfile::tempdir().unwrap();
    suno_in(tmp.path())
        .args(["skill", "install"])
        .assert()
        .code(0);

    let out = suno_in(tmp.path())
        .args(["skill", "status"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    for entry in json["data"].as_array().unwrap() {
        assert_eq!(entry["installed"], true);
        assert_eq!(entry["current"], true);
    }
}

#[test]
fn install_skill_back_compat_alias_routes() {
    let tmp = tempfile::tempdir().unwrap();
    suno_in(tmp.path()).arg("install-skill").assert().code(0);
}
