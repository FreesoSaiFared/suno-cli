//! Agent-skill management. The skill file is a signpost — trigger phrases in
//! the frontmatter, a body that points at `suno agent-info` — so capability
//! detail lives in the binary and the installed copies never drift.

use std::path::PathBuf;

use serde::Serialize;

use crate::errors::CliError;
use crate::output::OutputFormat;

pub const SKILL_CONTENT: &str = include_str!("../../assets/SKILL.md");

struct SkillTarget {
    platform: &'static str,
    dir: PathBuf,
}

fn home() -> PathBuf {
    // Prefer the HOME / USERPROFILE env vars over the platform home lookup:
    // `directories::UserDirs` reads the Known Folder API on Windows and ignores
    // HOME, so honoring the env var directly keeps tests hermetic there and
    // still resolves correctly for real users on every platform.
    for key in ["HOME", "USERPROFILE"] {
        if let Some(dir) = std::env::var_os(key).filter(|v| !v.is_empty()) {
            return PathBuf::from(dir);
        }
    }
    directories::UserDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Claude/Codex/Gemini always; ~/.agents only when that skills root already
/// exists (we install into platforms the user has, we don't create them).
fn skill_targets() -> Vec<SkillTarget> {
    let h = home();
    let mut targets = vec![
        SkillTarget {
            platform: "Claude Code",
            dir: h.join(".claude/skills/suno"),
        },
        SkillTarget {
            platform: "Codex CLI",
            dir: h.join(".codex/skills/suno"),
        },
        SkillTarget {
            platform: "Gemini CLI",
            dir: h.join(".gemini/skills/suno"),
        },
    ];
    if h.join(".agents/skills").is_dir() {
        targets.push(SkillTarget {
            platform: "Agents",
            dir: h.join(".agents/skills/suno"),
        });
    }
    targets
}

#[derive(Serialize)]
struct InstallResult {
    platform: &'static str,
    path: String,
    status: &'static str,
}

pub fn install(fmt: OutputFormat, quiet: bool, force: bool) -> Result<(), CliError> {
    let mut results = Vec::new();

    for target in &skill_targets() {
        let skill_path = target.dir.join("SKILL.md");
        let already_current = !force
            && skill_path.exists()
            && std::fs::read_to_string(&skill_path).is_ok_and(|c| c == SKILL_CONTENT);

        let status = if already_current {
            "already_current"
        } else {
            std::fs::create_dir_all(&target.dir)?;
            std::fs::write(&skill_path, SKILL_CONTENT)?;
            "installed"
        };
        results.push(InstallResult {
            platform: target.platform,
            path: skill_path.display().to_string(),
            status,
        });
    }

    match fmt {
        OutputFormat::Json => crate::output::json::success(&results),
        OutputFormat::Table => {
            if !quiet {
                for r in &results {
                    eprintln!("{}: {} ({})", r.platform, r.status, r.path);
                }
            }
        }
    }
    Ok(())
}

/// Back-compat `install-skill --path`: write a single copy to a custom
/// location (with ~ expansion) instead of the per-platform install. Refuses to
/// clobber an existing file unless `force`, so a mistyped path can't silently
/// overwrite an unrelated file; re-writing the identical skill is a no-op.
pub fn install_to_path(
    path: &str,
    fmt: OutputFormat,
    quiet: bool,
    force: bool,
) -> Result<(), CliError> {
    let dest: PathBuf = if let Some(stripped) = path.strip_prefix("~/") {
        home().join(stripped)
    } else {
        PathBuf::from(path)
    };

    let mut already_current = false;
    if dest.exists() {
        if !dest.is_file() {
            return Err(CliError::InvalidInput(format!(
                "{} exists and is not a regular file",
                dest.display()
            )));
        }
        already_current = std::fs::read_to_string(&dest).is_ok_and(|c| c == SKILL_CONTENT);
        if !already_current && !force {
            return Err(CliError::InvalidInput(format!(
                "{} already exists — pass --force to overwrite it",
                dest.display()
            )));
        }
    }

    if !already_current {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        atomic_write(&dest, SKILL_CONTENT)?;
    }

    match fmt {
        OutputFormat::Json => crate::output::json::success(serde_json::json!({
            "installed": true,
            "path": dest.display().to_string(),
            "status": if already_current { "already_current" } else { "installed" },
        })),
        OutputFormat::Table => {
            if !quiet {
                let verb = if already_current {
                    "Already current"
                } else {
                    "Installed suno skill to"
                };
                eprintln!("{verb}: {}", dest.display());
            }
        }
    }
    Ok(())
}

/// Write via a same-directory temp file + atomic rename so a crash or a
/// concurrent reader never sees a half-written skill file.
fn atomic_write(dest: &std::path::Path, contents: &str) -> Result<(), CliError> {
    let file_name = dest
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("SKILL.md");
    let tmp = dest.with_file_name(format!(".{file_name}.{}.tmp", std::process::id()));
    std::fs::write(&tmp, contents)?;
    if let Err(e) = std::fs::rename(&tmp, dest) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.into());
    }
    Ok(())
}

#[derive(Serialize)]
struct SkillStatus {
    platform: &'static str,
    path: String,
    installed: bool,
    current: bool,
}

pub fn status(fmt: OutputFormat) -> Result<(), CliError> {
    let results: Vec<SkillStatus> = skill_targets()
        .iter()
        .map(|target| {
            let skill_path = target.dir.join("SKILL.md");
            let installed = skill_path.exists();
            let current =
                installed && std::fs::read_to_string(&skill_path).is_ok_and(|c| c == SKILL_CONTENT);
            SkillStatus {
                platform: target.platform,
                path: skill_path.display().to_string(),
                installed,
                current,
            }
        })
        .collect();

    match fmt {
        OutputFormat::Json => crate::output::json::success(&results),
        OutputFormat::Table => {
            for r in &results {
                let state = match (r.installed, r.current) {
                    (true, true) => "current",
                    (true, false) => "outdated — run `suno skill install`",
                    _ => "not installed",
                };
                println!("{:<12} {state}", r.platform);
            }
        }
    }
    Ok(())
}
