//! `suno write` end-to-end invariants. The central one: the file named by the
//! emitted generate command exists, is directly consumable by
//! `generate --lyrics-file`, carries nothing that would be sung as lyrics, and
//! reflects every selected control.
//!
//! `write` reads neither config nor auth nor network, so the scrubbed `suno()`
//! helper is enough; only the placeholder preflight needs an isolated home.

mod common;
use common::{suno, suno_in};
use tempfile::TempDir;

fn write_json(args: &[&str]) -> serde_json::Value {
    let out = suno()
        .arg("write")
        .args(args)
        .arg("--json")
        .output()
        .unwrap();
    assert!(out.status.success(), "write failed: {args:?}");
    serde_json::from_slice(&out.stdout).expect("valid JSON envelope")
}

#[test]
fn out_file_is_lyrics_only_and_matches_the_envelope() {
    let tmp = TempDir::new().unwrap();
    let song = tmp.path().join("song.txt");
    let path = song.to_str().unwrap();
    let v = write_json(&["--genre", "pop", "--title", "Golden Hour", "--out", path]);

    let contents = std::fs::read_to_string(&song).unwrap();
    assert!(contents.contains("[Chorus]"));
    // Nothing that would be handed to Suno as lyrics and sung.
    for banned in ["Style Prompt", "Suno Tags", "Golden Hour", "---"] {
        assert!(
            !contents.contains(banned),
            "--out must be lyrics-only, found '{banned}'"
        );
    }
    // The envelope's `structure` is byte-identical to what landed on disk.
    assert_eq!(v["data"]["structure"].as_str().unwrap(), contents);
    assert_eq!(v["data"]["written"].as_str().unwrap(), path);
    // The metadata lives in the envelope instead.
    assert_eq!(v["data"]["title"], "Golden Hour");
    assert!(!v["data"]["style_prompt"].as_str().unwrap().is_empty());
}

#[test]
fn project_out_carries_the_composite_document() {
    let tmp = TempDir::new().unwrap();
    let lyrics = tmp.path().join("song.txt");
    let project = tmp.path().join("project.md");
    write_json(&[
        "--genre",
        "pop",
        "--title",
        "Golden Hour",
        "--out",
        lyrics.to_str().unwrap(),
        "--project-out",
        project.to_str().unwrap(),
    ]);
    let doc = std::fs::read_to_string(&project).unwrap();
    for anchor in ["Golden Hour", "Style Prompt", "Suno Tags", "[Chorus]"] {
        assert!(doc.contains(anchor), "project doc missing {anchor}");
    }
}

#[test]
fn next_action_points_at_the_real_file_and_is_absent_without_one() {
    let tmp = TempDir::new().unwrap();
    let song = tmp.path().join("out-song.txt");
    let path = song.to_str().unwrap();
    let v = write_json(&[
        "--genre",
        "pop",
        "--title",
        r#"She Said "Go""#,
        "--out",
        path,
    ]);
    let argv: Vec<String> = serde_json::from_value(v["data"]["next_action"]["argv"].clone())
        .expect("next_action.argv must be a string array");
    let i = argv.iter().position(|a| a == "--lyrics-file").unwrap();
    assert_eq!(argv[i + 1], path);
    assert!(std::path::Path::new(&argv[i + 1]).exists());
    let cmd = v["data"]["next_action"]["command"].as_str().unwrap();
    assert!(cmd.contains(r#"'She Said "Go"'"#), "unescaped title: {cmd}");
    assert!(!cmd.contains("song.txt --model"));

    // Without --out there is no file, so no runnable command may be emitted.
    let v = write_json(&["--genre", "pop"]);
    assert!(v["data"]["next_action"].is_null());
    assert_eq!(v["data"]["ready_to_generate"], false);
    assert!(
        v["data"]["missing_requirements"][0]
            .as_str()
            .unwrap()
            .contains("--out")
    );
}

#[test]
fn overrides_reach_every_field() {
    let v = write_json(&[
        "--genre",
        "modern-pop", // Uplifting, 120 BPM, female by default
        "--mood",
        "dark and brooding",
        "--vocal",
        "male",
        "--bpm",
        "88",
    ]);
    let style = v["data"]["style_prompt"].as_str().unwrap();
    let structure = v["data"]["structure"].as_str().unwrap();
    let tags = v["data"]["suno_tags"].as_str().unwrap();
    assert!(style.contains("dark and brooding") && style.contains("88 BPM"));
    assert!(structure.contains("[Mood: Dark]") && structure.contains("[Male Vocal]"));
    assert!(!structure.to_lowercase().contains("uplifting"));
    assert!(tags.contains("dark") && !tags.contains("uplifting"));
    assert!(tags.contains("male vocals") && !tags.contains("female vocals"));
}

#[test]
fn instrumental_is_coherent() {
    let tmp = TempDir::new().unwrap();
    let beat = tmp.path().join("beat.txt");
    let v = write_json(&[
        "--genre",
        "lo-fi",
        "--instrumental",
        "--out",
        beat.to_str().unwrap(),
    ]);
    let contents = std::fs::read_to_string(&beat).unwrap();
    assert!(!contents.contains('<'), "no fill-me slots: {contents}");
    assert_eq!(v["data"]["placeholders_remaining"], 0);
    assert_eq!(v["data"]["ready_to_generate"], true);
    let argv: Vec<String> =
        serde_json::from_value(v["data"]["next_action"]["argv"].clone()).unwrap();
    assert!(argv.iter().any(|a| a == "--instrumental"));
}

#[test]
fn priming_without_consent_fields_exits_3() {
    let out = suno()
        .args(["write", "--mode", "priming", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let v: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(v["status"], "error");
    let msg = v["error"]["message"].as_str().unwrap();
    for flag in ["--target", "--objective", "--domain"] {
        assert!(msg.contains(flag), "{msg}");
    }
    // No scaffold and no handoff for an unusable request.
    assert!(out.stdout.is_empty());
}

#[test]
fn priming_with_consent_fields_shapes_the_scaffold() {
    let v = write_json(&[
        "--mode",
        "priming",
        "--target",
        "anonymised batch (n=40)",
        "--objective",
        "increase recall of brand X",
        "--domain",
        "marketing",
    ]);
    assert_eq!(v["data"]["mode"], "priming");
    assert_eq!(v["data"]["theme"], "increase recall of brand X");
    let structure = v["data"]["structure"].as_str().unwrap();
    assert!(structure.contains("increase recall of brand X"));
    // The research artefact belongs in the envelope, never in the lyrics.
    assert!(!structure.contains("Prime-Stack"));
    assert!(
        v["data"]["priming"]["prime_stack_map"]
            .as_str()
            .unwrap()
            .contains("Prime-Stack Map")
    );
}

#[test]
fn generate_refuses_unresolved_scaffold_placeholders() {
    // Credit protection: an unfilled scaffold must never reach the API. The
    // preflight runs before auth, so this needs no credentials.
    let tmp = TempDir::new().unwrap();
    let song = tmp.path().join("song.txt");
    suno_in(tmp.path())
        .args([
            "write",
            "--genre",
            "pop",
            "--out",
            song.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();

    let out = suno_in(tmp.path())
        .args([
            "generate",
            "--title",
            "Draft",
            "--tags",
            "pop",
            "--lyrics-file",
            song.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let v: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains("placeholder"), "{msg}");
    // Name the offending lines so the caller can fix them without guessing.
    assert!(msg.contains("line(s)") && msg.chars().any(|c| c.is_ascii_digit()));
}

#[test]
fn write_help_states_the_out_and_redirection_contract() {
    let out = suno().args(["write", "--help"]).output().unwrap();
    assert!(out.status.success());
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(help.contains("--out"));
    // The piped-vs-TTY contradiction the review flagged must stay fixed.
    assert!(!help.contains("Plain text to stdout"));
    assert!(help.to_lowercase().contains("redirection"));
}
