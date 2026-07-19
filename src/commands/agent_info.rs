//! Machine-readable capability manifest (agent-cli-framework shape).
//!
//! Always raw JSON, never wrapped in the envelope — an agent calling
//! agent-info is bootstrapping and this IS the schema definition.
//! Every command key must stay routable (`suno <key> --help` exits 0);
//! the conformance probe enforces it.

use clap::ValueEnum;
use serde_json::{Value, json};

use crate::cli::{ModelVersion, RemasterModel};

/// Model maps are generated from the clap enums so the manifest can never
/// drift from what `--model` actually accepts.
fn model_map() -> serde_json::Map<String, Value> {
    ModelVersion::value_variants()
        .iter()
        .map(|m| (m.display_name().to_string(), m.to_api_key().into()))
        .collect()
}

fn remaster_model_map() -> serde_json::Map<String, Value> {
    RemasterModel::value_variants()
        .iter()
        .map(|m| {
            let name = m.to_possible_value().expect("no skipped variants");
            (name.get_name().to_string(), m.to_api_key().into())
        })
        .collect()
}

/// One `json!` per command: a single macro invocation for the whole map
/// blows the macro recursion limit, and per-command blocks read better.
fn command_map() -> serde_json::Map<String, Value> {
    let models: Vec<String> = ModelVersion::value_variants()
        .iter()
        .map(|m| m.display_name().to_string())
        .collect();

    let model_option = json!({
        "name": "--model", "type": "string", "required": false,
        "default": "v5.5 (config `default_model`)", "values": models,
        "description": "Model version"
    });
    let wait_option = json!({
        "name": "--wait", "type": "bool", "required": false, "default": false,
        "description": "Block until generation completes (config `poll_timeout_secs` caps the wait)"
    });
    let download_option = json!({
        "name": "--download", "type": "string", "required": false,
        "description": "Download finished MP3s to this directory (lyrics embedded)"
    });
    let token_option = json!({
        "name": "--token", "type": "string", "required": false,
        "description": "Pre-solved hCaptcha token (skips preflight and solver)"
    });
    let no_captcha_option = json!({
        "name": "--no-captcha", "type": "bool", "required": false, "default": false,
        "description": "Never run the captcha solver (headless boxes supplying --token)"
    });
    let force_option = json!({
        "name": "--force", "type": "bool", "required": false, "default": false,
        "description": "Bypass the duplicate-run guard"
    });
    let clip_id_arg = json!({
        "name": "clip_id", "kind": "positional", "type": "string", "required": true,
        "description": "Clip ID"
    });
    let ids_arg = json!({
        "name": "ids", "kind": "positional", "type": "string...", "required": true,
        "description": "One or more clip IDs"
    });

    let commands: Vec<(&str, Value)> = vec![
        (
            "generate",
            json!({
                "description": "Generate music with custom lyrics, tags, and controls",
                "args": [],
                "options": [
                    {"name": "--title", "type": "string", "required": false, "description": "Song title"},
                    {"name": "--tags", "type": "string", "required": false, "description": "Style tags, comma-separated"},
                    {"name": "--exclude", "type": "string", "required": false, "description": "Styles to avoid"},
                    {"name": "--lyrics", "type": "string", "required": false, "description": "Lyrics text with [Verse]/[Chorus] tags"},
                    {"name": "--lyrics-file", "type": "string", "required": false, "description": "Read lyrics from file"},
                    model_option,
                    {"name": "--vocal", "type": "string", "required": false, "values": ["male", "female"], "description": "Vocal gender"},
                    {"name": "--weirdness", "type": "number", "required": false, "description": "0-100"},
                    {"name": "--style-influence", "type": "number", "required": false, "description": "0-100"},
                    {"name": "--audio-influence", "type": "number", "required": false, "description": "0-100"},
                    {"name": "--instrumental", "type": "bool", "required": false, "default": false, "description": "No vocals"},
                    {"name": "--persona", "type": "string", "required": false, "description": "Voice persona UUID"},
                    wait_option, download_option, token_option, no_captcha_option, force_option
                ]
            }),
        ),
        (
            "describe",
            json!({
                "description": "Generate music from a text description (Suno writes lyrics)",
                "args": [],
                "options": [
                    {"name": "--prompt", "type": "string", "required": true, "description": "What the song should be"},
                    {"name": "--tags", "type": "string", "required": false, "description": "Style tags"},
                    model_option,
                    {"name": "--vocal", "type": "string", "required": false, "values": ["male", "female"], "description": "Vocal gender"},
                    {"name": "--weirdness", "type": "number", "required": false, "description": "0-100"},
                    {"name": "--style-influence", "type": "number", "required": false, "description": "0-100"},
                    {"name": "--instrumental", "type": "bool", "required": false, "default": false, "description": "No vocals"},
                    {"name": "--persona", "type": "string", "required": false, "description": "Voice persona UUID"},
                    wait_option, download_option, token_option, no_captcha_option, force_option
                ]
            }),
        ),
        (
            "lyrics",
            json!({
                "description": "Generate lyrics only (free, no credits used)",
                "args": [],
                "options": [
                    {"name": "--prompt", "type": "string", "required": true, "description": "What the song should be about"}
                ]
            }),
        ),
        (
            "extend",
            json!({
                "description": "Continue/extend a clip from a timestamp",
                "args": [clip_id_arg],
                "options": [
                    {"name": "--at", "type": "number", "required": true, "description": "Timestamp in seconds to continue from"},
                    {"name": "--lyrics", "type": "string", "required": false, "description": "New lyrics for the extension"},
                    {"name": "--tags", "type": "string", "required": false, "description": "Style tags"},
                    model_option, wait_option, token_option, no_captcha_option, force_option
                ]
            }),
        ),
        (
            "concat",
            json!({
                "description": "Concatenate an extended clip into a full song",
                "args": [clip_id_arg],
                "options": []
            }),
        ),
        (
            "cover",
            json!({
                "description": "Create a cover of an existing clip",
                "args": [clip_id_arg],
                "options": [
                    {"name": "--tags", "type": "string", "required": false, "description": "Style tags for the cover"},
                    model_option,
                    {"name": "--audio-influence", "type": "number", "required": false, "description": "0-100, how strongly the source clip shapes the cover"},
                    wait_option, download_option, token_option, no_captcha_option, force_option
                ]
            }),
        ),
        (
            "remaster",
            json!({
                "description": "Remaster a clip with a different model",
                "args": [clip_id_arg],
                "options": [
                    {"name": "--model", "type": "string", "required": false, "default": "v5.5",
                     "values": remaster_model_map().keys().cloned().collect::<Vec<_>>(),
                     "description": "Remaster model version"},
                    wait_option, download_option, token_option, no_captcha_option, force_option
                ]
            }),
        ),
        (
            "stems",
            json!({
                "description": "Extract stems (vocals, instruments) from a clip",
                "args": [clip_id_arg],
                "options": []
            }),
        ),
        (
            "info",
            json!({
                "description": "Show detailed info for a single clip",
                "args": [{"name": "id", "kind": "positional", "type": "string", "required": true, "description": "Clip ID"}],
                "options": []
            }),
        ),
        (
            "persona",
            json!({
                "description": "View a voice persona",
                "args": [{"name": "id", "kind": "positional", "type": "string", "required": true, "description": "Persona ID"}],
                "options": []
            }),
        ),
        (
            "list",
            json!({
                "description": "List your songs (JSON data: {clips, next_cursor, has_more})",
                "aliases": ["ls"],
                "args": [],
                "options": [
                    {"name": "--cursor", "type": "string", "required": false, "description": "Opaque next_cursor token from a previous page"}
                ]
            }),
        ),
        (
            "search",
            json!({
                "description": "Search your songs by title or tags",
                "args": [{"name": "query", "kind": "positional", "type": "string", "required": true, "description": "Search query"}],
                "options": []
            }),
        ),
        (
            "status",
            json!({
                "description": "Check generation status",
                "args": [ids_arg],
                "options": []
            }),
        ),
        (
            "download",
            json!({
                "description": "Download audio/video for clip(s), embedding lyrics into MP3s",
                "aliases": ["dl"],
                "args": [ids_arg],
                "options": [
                    {"name": "--output", "type": "string", "required": false, "default": ". (config `output_dir`)", "description": "Output directory"},
                    {"name": "--video", "type": "bool", "required": false, "default": false, "description": "Download video instead of audio"}
                ]
            }),
        ),
        (
            "delete",
            json!({
                "description": "Delete/trash clip(s); requires -y (no interactive confirmation)",
                "aliases": ["rm"],
                "args": [ids_arg],
                "options": [
                    {"name": "--yes", "type": "bool", "required": true, "default": false, "description": "Confirm deletion (-y)"}
                ]
            }),
        ),
        (
            "set",
            json!({
                "description": "Update clip title, lyrics, or caption",
                "args": [{"name": "id", "kind": "positional", "type": "string", "required": true, "description": "Clip ID"}],
                "options": [
                    {"name": "--title", "type": "string", "required": false, "description": "New title"},
                    {"name": "--lyrics", "type": "string", "required": false, "description": "New lyrics"},
                    {"name": "--lyrics-file", "type": "string", "required": false, "description": "Read lyrics from file"},
                    {"name": "--caption", "type": "string", "required": false, "description": "New caption"},
                    {"name": "--remove-cover", "type": "bool", "required": false, "default": false, "description": "Remove custom cover image"}
                ]
            }),
        ),
        (
            "publish",
            json!({
                "description": "Toggle clip public/private",
                "args": [ids_arg],
                "options": [
                    {"name": "--private", "type": "bool", "required": false, "default": false, "description": "Make private instead of public"}
                ]
            }),
        ),
        (
            "timed-lyrics",
            json!({
                "description": "Word-level timestamped lyrics",
                "args": [{"name": "id", "kind": "positional", "type": "string", "required": true, "description": "Clip ID"}],
                "options": [
                    {"name": "--lrc", "type": "bool", "required": false, "default": false,
                     "description": "Raw LRC on stdout — stays raw even when piped (documented envelope exception)"}
                ]
            }),
        ),
        (
            "credits",
            json!({
                "description": "Show credit balance and plan info",
                "args": [],
                "options": []
            }),
        ),
        (
            "models",
            json!({
                "description": "List available models (live from your plan)",
                "args": [],
                "options": []
            }),
        ),
        (
            "auth",
            json!({
                "description": "Set up authentication (browser extract, cookie, JWT, refresh, logout)",
                "args": [],
                "options": [
                    {"name": "--login", "type": "bool", "required": false, "default": false, "description": "Auto-extract from browser (recommended)"},
                    {"name": "--refresh", "type": "bool", "required": false, "default": false, "description": "Force-refresh the JWT via stored Clerk session"},
                    {"name": "--cookie", "type": "string", "required": false, "description": "Cookie header or raw __client value"},
                    {"name": "--jwt", "type": "string", "required": false, "description": "Direct JWT (~1h lifetime)"},
                    {"name": "--device", "type": "string", "required": false, "description": "Device ID override"},
                    {"name": "--logout", "type": "bool", "required": false, "default": false, "description": "Remove stored authentication"}
                ]
            }),
        ),
        (
            "config show",
            json!({
                "description": "Show the effective merged configuration",
                "args": [],
                "options": []
            }),
        ),
        (
            "config set",
            json!({
                "description": "Set a configuration value in the config file",
                "args": [
                    {"name": "key", "kind": "positional", "type": "string", "required": true,
                     "description": crate::config::CONFIG_KEYS.join(" | ")},
                    {"name": "value", "kind": "positional", "type": "string", "required": true, "description": "New value"}
                ],
                "options": []
            }),
        ),
        (
            "config path",
            json!({
                "description": "Show the configuration file path",
                "args": [],
                "options": []
            }),
        ),
        (
            "config check",
            json!({
                "description": "Validate the configuration file",
                "args": [],
                "options": []
            }),
        ),
        (
            "doctor",
            json!({
                "description": "Check auth, Chrome, network, credits, captcha state, and config health",
                "args": [],
                "options": [],
                "exit_behavior": "0 if no check fails (warnings allowed), 2 if any check fails"
            }),
        ),
        (
            "agent-info",
            json!({
                "description": "This manifest",
                "args": [],
                "options": []
            }),
        ),
        (
            "skill install",
            json!({
                "description": "Install the agent skill to all detected platforms (idempotent)",
                "args": [],
                "options": []
            }),
        ),
        (
            "skill status",
            json!({
                "description": "Check which platforms have the skill installed and current",
                "args": [],
                "options": []
            }),
        ),
        (
            "update",
            json!({
                "description": "Distribution-aware update check/apply",
                "args": [],
                "options": [
                    {"name": "--check", "type": "bool", "required": false, "default": false, "description": "Check only, don't install"},
                    force_option
                ],
                "install_sources": ["standalone", "homebrew", "cargo"],
                "data_fields": [
                    "current_version", "latest_version", "status", "install_source",
                    "update_mode", "upgrade_command", "release_url", "requires_skill_reinstall"
                ]
            }),
        ),
    ];

    commands
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect()
}

pub fn run() {
    let auth_path = crate::config::config_dir()
        .join("auth.json")
        .display()
        .to_string();

    let info = json!({
        "name": "suno",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Suno AI music generation CLI — v5.5 with voice personas, covers, remasters",
        "commands": command_map(),
        "global_flags": {
            "--json": {
                "description": "Force JSON output (auto-enabled when piped)",
                "type": "bool",
                "default": false
            },
            "--quiet": {
                "description": "Suppress informational stderr output",
                "type": "bool",
                "default": false
            }
        },
        "exit_codes": {
            "0": "Success",
            "1": "Transient error (network, API, download) — retry with backoff",
            "2": "Configuration or auth error — run `suno doctor`; for auth run `suno auth --login`",
            "3": "Bad input (arguments, unknown ID, moderation rejection, duplicate run) — fix before retrying",
            "4": "Rate limited — wait 30-60s and retry",
        },
        "breaking_changes": {
            "0.6.0": "Exit codes remapped to the framework contract: auth errors 3→2, not-found 5→3, code 5 removed. `list --json` data is now {clips, next_cursor, has_more}; `list --page` → `--cursor`; `generate --variation` removed."
        },
        "envelope": {
            "version": "1",
            "success": "{ version, status, data }",
            "error": "{ version, status, error: { code, message, suggestion } }",
            "statuses": ["success", "no_results", "partial_success", "error"],
            "raw_output_exceptions": [
                "agent-info (this manifest)",
                "timed-lyrics --lrc (raw LRC on stdout even when piped)"
            ]
        },
        "config": {
            "path": crate::config::config_path().display().to_string(),
            "env_prefix": "SUNO_",
            "keys": {
                "default_model": "Default --model for generate/describe/extend/cover (clap name, e.g. v5.5)",
                "poll_interval_secs": "Initial poll backoff for --wait (doubles up to 15s)",
                "poll_timeout_secs": "Total --wait timeout before giving up",
                "output_dir": "Default directory for `download`",
            },
            "precedence": "flag > SUNO_* env > config file > default"
        },
        "auto_json_when_piped": true,
        // Domain extras below (schema allows additionalProperties).
        // Generated from the --model clap enums, so they cannot drift.
        "models": model_map(),
        "remaster_models": remaster_model_map(),
        "generation_cost": {
            "v5.5": "~70 credits per call (35 per clip, 2 clips)",
            "note": "Older models cost less. `lyrics` is free.",
        },
        "features": [
            "tags", "negative_tags", "vocal_gender",
            "weirdness", "style_influence", "audio_influence",
            "instrumental", "extend", "concat", "cover", "remaster",
            "stems", "lyrics", "timed_lyrics", "set_metadata",
            "set_visibility", "search", "delete", "captcha_check",
            "id3_lyrics_embedding", "voice_persona", "clip_info"
        ],
        "auth_path": auth_path,
        "auth": {
            "recommended": "suno auth --login",
            "methods": [
                "browser_cookie_extract",
                "full_cookie_header",
                "raw_clerk_client_cookie",
                "direct_jwt",
                "stored_clerk_refresh",
            ],
            "logout": "suno auth --logout",
            "generation_captcha": "Generation preflights /api/c/check and skips the solver when the account is not captcha-gated (the common case). When gated, the browser-backed hCaptcha solver runs automatically. Use --no-captcha only with a valid --token or for deliberate API tests.",
        },
        "provider": "direct_suno_unofficial",
        "auth_required": true,
        "default_model": "chirp-fenix (v5.5)",
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&info).unwrap_or_default()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_maps_mirror_the_clap_enums() {
        let m = model_map();
        assert_eq!(m.len(), ModelVersion::value_variants().len());
        assert_eq!(m["v4.5-all"], "chirp-auk-turbo");
        let r = remaster_model_map();
        assert_eq!(r.len(), RemasterModel::value_variants().len());
        assert_eq!(r["v5.5"], "chirp-flounder");
    }
}
