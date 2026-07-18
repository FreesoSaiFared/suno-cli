//! Distribution-aware update (framework update-standard): a binary installed
//! by Homebrew or Cargo is upgraded by that package manager — self-replace is
//! only for standalone GitHub-release installs. Overwriting a brew-owned
//! binary with a raw release asset is the failure mode this prevents.

use serde::Serialize;
use std::path::Path;

use crate::errors::CliError;
use crate::output::OutputFormat;

const REPO_OWNER: &str = "paperfoot";
const REPO_NAME: &str = "suno-cli";
const BREW_FORMULA: &str = "paperfoot/tap/suno";

#[derive(Serialize)]
struct UpdateResult {
    current_version: &'static str,
    latest_version: String,
    status: &'static str,
    install_source: &'static str,
    update_mode: &'static str,
    upgrade_command: Option<String>,
    release_url: String,
    requires_skill_reinstall: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum InstallSource {
    Homebrew,
    Cargo,
    Standalone,
    /// Provenance we can't classify. Self-update is disabled here (fail closed)
    /// so we never overwrite a package-manager binary we failed to recognize.
    Unknown,
}

impl InstallSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Homebrew => "homebrew",
            Self::Cargo => "cargo",
            Self::Standalone => "standalone",
            Self::Unknown => "unknown",
        }
    }
}

/// Exe-path heuristics; SUNO_INSTALL_SOURCE overrides for tests and for
/// installs the heuristics can't see (e.g. brew --prefix on a custom mount).
fn detect_install_source() -> Result<InstallSource, CliError> {
    if let Ok(raw) = std::env::var("SUNO_INSTALL_SOURCE") {
        return match raw.trim().to_ascii_lowercase().as_str() {
            "homebrew" | "brew" => Ok(InstallSource::Homebrew),
            "cargo" => Ok(InstallSource::Cargo),
            "standalone" => Ok(InstallSource::Standalone),
            "unknown" => Ok(InstallSource::Unknown),
            other => Err(CliError::Config(format!(
                "invalid SUNO_INSTALL_SOURCE '{other}' (expected homebrew, cargo, standalone, or unknown)"
            ))),
        };
    }

    let exe = std::env::current_exe()?;
    let path = exe.to_string_lossy();

    if path.contains("/Cellar/") || path.starts_with("/opt/homebrew/bin/") {
        return Ok(InstallSource::Homebrew);
    }

    let cargo_bin = std::env::var_os("CARGO_HOME")
        .map(|p| Path::new(&p).join("bin"))
        .or_else(|| std::env::var_os("HOME").map(|h| Path::new(&h).join(".cargo/bin")));
    if let Some(cargo_bin) = cargo_bin
        && exe.starts_with(cargo_bin)
    {
        return Ok(InstallSource::Cargo);
    }

    Ok(InstallSource::Standalone)
}

fn print_result(result: &UpdateResult, fmt: OutputFormat, quiet: bool) {
    match fmt {
        OutputFormat::Json => crate::output::json::success(result),
        OutputFormat::Table => {
            if quiet {
                return;
            }
            match result.status {
                "managed_install" => {
                    eprintln!("Installed via {}", result.install_source);
                    if let Some(cmd) = &result.upgrade_command {
                        eprintln!("Update with: {cmd}");
                    }
                }
                "up_to_date" => eprintln!("Up to date (v{})", result.current_version),
                "update_available" => {
                    eprintln!(
                        "Update available: v{} -> v{}",
                        result.current_version, result.latest_version
                    );
                    eprintln!("Run `suno update` to install");
                }
                _ => {
                    eprintln!(
                        "Updated: v{} -> v{}",
                        result.current_version, result.latest_version
                    );
                    eprintln!("Run `suno skill install` to refresh the agent skill");
                }
            }
        }
    }
}

pub fn run(check: bool, force: bool, fmt: OutputFormat, quiet: bool) -> Result<(), CliError> {
    let current = env!("CARGO_PKG_VERSION");
    let source = detect_install_source()?;

    if source == InstallSource::Unknown {
        // Fail closed: we can't tell whether a self-replace would clobber a
        // package-manager-owned binary, so refuse rather than risk it. Exit 2
        // (setup fix), with the known reinstall channels in the message.
        return Err(CliError::Config(
            "cannot determine how suno was installed — refusing to self-update so a \
             package-manager-owned binary is never overwritten. Reinstall from a known \
             channel: `brew install paperfoot/tap/suno`, `cargo install --locked suno`, \
             or download the release binary from \
             https://github.com/paperfoot/suno-cli/releases"
                .into(),
        ));
    }

    if source != InstallSource::Standalone {
        // Package-manager-owned binary: never self-replace, hand back the
        // owner channel's upgrade command instead.
        let upgrade_command = match source {
            InstallSource::Homebrew => format!("brew upgrade {BREW_FORMULA}"),
            _ => "cargo install --locked --force suno".to_string(),
        };
        let result = UpdateResult {
            current_version: current,
            latest_version: current.to_string(),
            status: "managed_install",
            install_source: source.as_str(),
            update_mode: "package_manager",
            upgrade_command: Some(upgrade_command),
            release_url: format!("https://github.com/{REPO_OWNER}/{REPO_NAME}/releases"),
            requires_skill_reinstall: true,
        };
        print_result(&result, fmt, quiet);
        return Ok(());
    }

    let updater = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name("suno")
        // self_update defaults chatter release info to stdout and prompt on
        // stdin — both break the envelope contract (stdout carries only the
        // JSON envelope; piped consumers exiting early would EPIPE-panic the
        // chatter). Download progress draws to stderr and may stay.
        .show_output(false)
        .no_confirm(true)
        .show_download_progress(!quiet)
        .current_version(current)
        .build()
        .map_err(|e| CliError::Update(e.to_string()))?;

    let (latest, status) = if check {
        let latest = updater
            .get_latest_release()
            .map_err(|e| CliError::Update(e.to_string()))?;
        let v = latest.version.trim_start_matches('v').to_string();
        // Semantic compare, not string equality: a local build ahead of the
        // latest release (0.6.0 vs 0.5.7) must read up_to_date, never
        // update_available.
        let status = if self_update::version::bump_is_greater(current, &v)
            .map_err(|e| CliError::Update(e.to_string()))?
        {
            "update_available"
        } else {
            "up_to_date"
        };
        (v, status)
    } else {
        // Self-replacement must not run twice concurrently (agent retries).
        let mut guard = crate::guard::DuplicateGuard::new(&crate::config::data_dir(), "update");
        guard.acquire(force)?;

        let release = updater
            .update()
            .map_err(|e| CliError::Update(e.to_string()))?;
        let v = release.version().trim_start_matches('v').to_string();
        let status = if self_update::version::bump_is_greater(current, &v)
            .map_err(|e| CliError::Update(e.to_string()))?
        {
            "updated"
        } else {
            "up_to_date"
        };
        (v, status)
    };

    let result = UpdateResult {
        current_version: current,
        release_url: format!("https://github.com/{REPO_OWNER}/{REPO_NAME}/releases/tag/v{latest}"),
        latest_version: latest,
        status,
        install_source: source.as_str(),
        update_mode: "self_replace",
        upgrade_command: None,
        requires_skill_reinstall: status != "up_to_date",
    };
    print_result(&result, fmt, quiet);
    Ok(())
}
