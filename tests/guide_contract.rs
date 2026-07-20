//! The built-in guides system: list, emit, alias resolution, the
//! unknown-name error, and coherence between the agent-info `guides` array
//! and what `suno guide` actually serves (single source of truth — no drift).
//!
//! `guide` reads neither config nor auth nor network, so the scrubbed `suno()`
//! (no home isolation) is enough.

mod common;
use common::{agent_info, suno};
use std::collections::BTreeSet;

fn served_names() -> BTreeSet<String> {
    let out = suno().args(["guide", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    json["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn list_returns_both_guides_json() {
    let out = suno().args(["guide", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["status"], "success");

    let entries = json["data"].as_array().unwrap();
    let names: BTreeSet<&str> = entries
        .iter()
        .map(|g| g["name"].as_str().unwrap())
        .collect();
    assert!(names.contains("songwriting"));
    assert!(names.contains("priming"));

    // Every listed guide carries a description and an aliases array.
    for g in entries {
        assert!(!g["description"].as_str().unwrap().is_empty());
        assert!(g["aliases"].is_array());
    }
}

#[test]
fn songwriting_guide_emits_markdown_with_anchors() {
    let out = suno().args(["guide", "songwriting"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["data"]["name"], "songwriting");

    let content = json["data"]["content"].as_str().unwrap();
    assert!(!content.trim().is_empty());
    assert!(
        content.contains("[Chorus]"),
        "songwriting must show structure tags"
    );
    assert!(
        content.contains("Style Prompt"),
        "songwriting must cover the Style Prompt"
    );
}

#[test]
fn priming_guide_emits_markdown_with_anchors() {
    let out = suno().args(["guide", "priming"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["data"]["name"], "priming");

    let content = json["data"]["content"].as_str().unwrap().to_lowercase();
    assert!(
        content.contains("evidence grade"),
        "priming must grade evidence"
    );
    assert!(
        content.contains("consent"),
        "priming must foreground consent"
    );
}

#[test]
fn alias_resolves_to_canonical_guide() {
    let canonical = suno().args(["guide", "songwriting"]).output().unwrap();
    let via_alias = suno().args(["guide", "grammar"]).output().unwrap();
    assert_eq!(canonical.status.code(), Some(0));
    assert_eq!(via_alias.status.code(), Some(0));
    // Same bytes: the alias must resolve to the exact same guide.
    assert_eq!(canonical.stdout, via_alias.stdout);
}

#[test]
fn unknown_guide_exits_3_and_lists_available_names() {
    let out = suno().args(["guide", "nonexistent"]).output().unwrap();
    assert_eq!(out.status.code(), Some(3));

    // Piped → JSON error envelope on stderr, naming the available guides.
    let stderr = String::from_utf8_lossy(&out.stderr);
    let json: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("error envelope must be JSON");
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "invalid_input");
    let msg = json["error"]["message"].as_str().unwrap();
    assert!(msg.contains("songwriting") && msg.contains("priming"));
}

#[test]
fn guides_alias_command_lists() {
    // The `guides` visible alias also reaches the list.
    let out = suno().args(["guides", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(!json["data"].as_array().unwrap().is_empty());
}

#[test]
fn agent_info_guides_array_matches_the_registry() {
    let info = agent_info();
    let guides = info["guides"]
        .as_array()
        .expect("agent-info must expose a non-empty guides array");
    assert!(!guides.is_empty());
    for g in guides {
        assert!(!g["name"].as_str().unwrap().is_empty());
        assert!(!g["description"].as_str().unwrap().is_empty());
    }

    let info_names: BTreeSet<String> = guides
        .iter()
        .map(|g| g["name"].as_str().unwrap().to_string())
        .collect();

    // Coherence: the manifest's guides list must equal what `suno guide` serves,
    // and every advertised guide must actually emit.
    assert_eq!(info_names, served_names());
    for name in &info_names {
        let out = suno().args(["guide", name]).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "advertised guide {name} must emit"
        );
    }
}
