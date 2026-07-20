//! Built-in guides: the CLI ships its own songwriting knowledge as
//! discoverable docs, so agents read a single source of truth compiled into
//! the binary instead of a skill copy that drifts. Adding a guide is one
//! `GUIDES` entry — name, aliases, description, and an `include_str!` of the
//! markdown under assets/guides/.

use serde::Serialize;

use crate::errors::CliError;
use crate::output::OutputFormat;

pub struct Guide {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub content: &'static str,
}

/// The registry. One array entry per guide; everything else (listing,
/// alias resolution, the agent-info `guides` array) derives from it.
pub const GUIDES: &[Guide] = &[
    Guide {
        name: "songwriting",
        // No `write` alias: `suno write` is a command, and `suno guide write`
        // reading as a synonym for it confused the composer with its manual.
        aliases: &["grammar", "songwriting-guide"],
        description: "How to write for Suno: structure, meta-tags, genres, vocal styles, hooks (the base grammar)",
        content: include_str!("../../assets/guides/songwriting.md"),
    },
    Guide {
        name: "priming",
        aliases: &["priming-songs", "prime"],
        description: "Research/priming songs: evidence-graded psychological priming woven into lyrics (consent-based)",
        content: include_str!("../../assets/guides/priming.md"),
    },
];

/// Resolve a name or alias (case-insensitive) to a guide.
fn resolve(name: &str) -> Option<&'static Guide> {
    GUIDES.iter().find(|g| {
        g.name.eq_ignore_ascii_case(name) || g.aliases.iter().any(|a| a.eq_ignore_ascii_case(name))
    })
}

#[derive(Serialize)]
struct GuideSummary {
    name: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
}

/// `suno guide` with no name lists; with a name prints that guide's raw
/// markdown to stdout (a documented envelope exception, like
/// `timed-lyrics --lrc`).
pub fn run(name: Option<String>, fmt: OutputFormat, quiet: bool) -> Result<(), CliError> {
    match name {
        None => list(fmt, quiet),
        Some(name) => emit(&name, fmt),
    }
}

fn list(fmt: OutputFormat, quiet: bool) -> Result<(), CliError> {
    match fmt {
        OutputFormat::Json => {
            let summaries: Vec<GuideSummary> = GUIDES
                .iter()
                .map(|g| GuideSummary {
                    name: g.name,
                    aliases: g.aliases,
                    description: g.description,
                })
                .collect();
            crate::output::json::success(&summaries);
        }
        OutputFormat::Table => {
            for g in GUIDES {
                println!("{:<14} {}", g.name, g.description);
                if !g.aliases.is_empty() {
                    println!("{:<14} aliases: {}", "", g.aliases.join(", "));
                }
            }
            if !quiet {
                eprintln!("\nRead one with: suno guide <name>");
            }
        }
    }
    Ok(())
}

fn emit(name: &str, fmt: OutputFormat) -> Result<(), CliError> {
    let guide = resolve(name).ok_or_else(|| {
        let names: Vec<&str> = GUIDES.iter().map(|g| g.name).collect();
        CliError::InvalidInput(format!(
            "unknown guide '{name}' — available guides: {}",
            names.join(", ")
        ))
    })?;

    match fmt {
        OutputFormat::Json => crate::output::json::success(serde_json::json!({
            "name": guide.name,
            "content": guide.content,
        })),
        // Raw markdown to stdout, verbatim — the content already ends with a
        // newline, so `print!` keeps it byte-for-byte.
        OutputFormat::Table => print!("{}", guide.content),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_aliases_resolve() {
        assert_eq!(resolve("songwriting").unwrap().name, "songwriting");
        assert_eq!(resolve("grammar").unwrap().name, "songwriting");
        assert_eq!(resolve("PRIME").unwrap().name, "priming");
        assert!(resolve("nope").is_none());
    }

    #[test]
    fn every_guide_has_nonempty_content() {
        for g in GUIDES {
            assert!(!g.name.is_empty());
            assert!(!g.description.is_empty());
            assert!(!g.content.trim().is_empty(), "{} is empty", g.name);
        }
    }
}
