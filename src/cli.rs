use clap::{Parser, Subcommand, ValueEnum};

// Agents read --help to bootstrap usage; keep tips short and examples real.
const HELP_FOOTER: &str = "\
Tips:
  • First run: `suno auth --login`, then `suno doctor` to verify the setup
  • Output is a JSON envelope automatically when piped; force with --json
  • `suno lyrics` is free; generation costs ~70 credits per call on v5.5
  • Exit codes: 0 ok, 1 transient (retry), 2 config/auth, 3 bad input, 4 rate limited
  • Config: `suno config path` shows the file; SUNO_* env vars override it
  • Full machine-readable manifest: `suno agent-info | jq`

Examples:
  suno generate --title \"Weekend Code\" --tags \"indie rock, upbeat\" --lyrics-file lyrics.txt --wait --download ./songs/
    Generate a song and download the finished MP3 (lyrics embedded)

  suno describe --prompt \"a chill lo-fi track about rainy mornings\" --wait --download ./
    Let Suno write the lyrics from a description

  suno list | jq -r '.data.clips[].id'
    List your library as JSON and extract clip IDs

  suno timed-lyrics <clip_id> --lrc > song.lrc
    Word-level synced lyrics in LRC format (raw even when piped)

  suno cover <clip_id> --tags \"jazz, smooth piano\" --wait
    Re-imagine an existing clip in a new style";

#[derive(Parser)]
#[command(
    name = "suno",
    version,
    about = "Suno AI music generation CLI — v5.5 support",
    after_long_help = HELP_FOOTER
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output JSON (auto-detected when piped)
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress non-essential output
    #[arg(long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate music with custom lyrics, tags, and controls
    Generate(GenerateArgs),

    /// Generate music from a text description (Suno writes lyrics)
    Describe(DescribeArgs),

    /// Generate lyrics only (free, no credits used)
    Lyrics(LyricsArgs),

    /// Continue/extend a clip from a timestamp
    Extend(ExtendArgs),

    /// Concatenate clips into a full song
    Concat(ConcatArgs),

    /// Create a cover of an existing clip
    Cover(CoverArgs),

    /// Remaster a clip with a different model
    Remaster(RemasterArgs),

    /// Extract stems (vocals, instruments) from a clip
    Stems(StemsArgs),

    /// Show detailed info for a single clip
    Info(InfoArgs),

    /// View a voice persona
    Persona(PersonaArgs),

    /// List your songs
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// Search your songs by title or tags
    Search(SearchArgs),

    /// Check generation status
    Status(StatusArgs),

    /// Download audio/video for clip(s)
    #[command(visible_alias = "dl")]
    Download(DownloadArgs),

    /// Delete/trash a clip
    #[command(visible_alias = "rm")]
    Delete(DeleteArgs),

    /// Update clip title, lyrics, or caption
    Set(SetArgs),

    /// Toggle clip public/private
    Publish(PublishArgs),

    /// Get word-level timestamped lyrics
    TimedLyrics(TimedLyricsArgs),

    /// Show credit balance and plan info
    Credits,

    /// List available models
    Models,

    /// Set up authentication
    Auth(AuthArgs),

    /// Manage configuration
    Config(ConfigArgs),

    /// Check external dependencies and configuration health
    Doctor,

    /// Machine-readable capabilities (for AI agents)
    AgentInfo,

    /// Manage the agent skill (teaches Claude Code / Codex / Gemini how to use this CLI)
    Skill(SkillArgs),

    /// Back-compat alias for `skill install` (hidden)
    #[command(hide = true)]
    InstallSkill(InstallSkillArgs),

    /// Distribution-aware update check/apply
    Update(UpdateArgs),

    /// Hidden: deterministic exit-code trigger for contract tests
    #[command(hide = true)]
    Contract {
        /// Exit code to trigger (0-4)
        code: i32,
    },
}

#[derive(clap::Args)]
pub struct UpdateArgs {
    /// Check for a new version without installing
    #[arg(long)]
    pub check: bool,

    /// Bypass the duplicate-run guard
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::Args)]
pub struct SkillArgs {
    #[command(subcommand)]
    pub action: SkillAction,
}

#[derive(Subcommand)]
pub enum SkillAction {
    /// Write the skill file to all detected agent platforms
    Install,
    /// Check which platforms have the skill installed and current
    Status,
}

#[derive(clap::Args)]
pub struct GenerateArgs {
    /// Song title
    #[arg(short, long)]
    pub title: Option<String>,

    /// Style tags (comma-separated): "pop, synths, upbeat"
    #[arg(long)]
    pub tags: Option<String>,

    /// Exclude styles (comma-separated): "metal, heavy"
    #[arg(long)]
    pub exclude: Option<String>,

    /// Lyrics text (with [Verse], [Chorus] tags)
    #[arg(short, long, conflicts_with = "lyrics_file")]
    pub lyrics: Option<String>,

    /// Read lyrics from file
    #[arg(long)]
    pub lyrics_file: Option<String>,

    /// Model version (default: config `default_model`, v5.5 out of the box)
    #[arg(short, long)]
    pub model: Option<ModelVersion>,

    /// Vocal gender
    #[arg(long)]
    pub vocal: Option<VocalGender>,

    /// Weirdness level (0-100)
    #[arg(long)]
    pub weirdness: Option<f64>,

    /// Style influence strength (0-100)
    #[arg(long)]
    pub style_influence: Option<f64>,

    /// Audio influence strength (0-100) — how strongly source audio shapes
    /// the output
    #[arg(long)]
    pub audio_influence: Option<f64>,

    /// Generate instrumental only (no vocals)
    #[arg(long)]
    pub instrumental: bool,

    /// Bypass the duplicate-run guard
    #[arg(long)]
    pub force: bool,

    /// Wait for generation to complete
    #[arg(short, long)]
    pub wait: bool,

    /// Download output to directory after generation
    #[arg(long)]
    pub download: Option<String>,

    /// hCaptcha token (overrides the auto-solver)
    #[arg(long)]
    pub token: Option<String>,

    /// Skip the built-in hCaptcha auto-solver. Useful for headless servers
    /// where you supply --token directly (e.g. from a 2Captcha solution).
    #[arg(long)]
    pub no_captcha: bool,

    /// Voice persona ID (generates with your custom voice)
    #[arg(long)]
    pub persona: Option<String>,
}

#[derive(clap::Args)]
pub struct DescribeArgs {
    /// Description of the song you want
    #[arg(short, long)]
    pub prompt: String,

    /// Style tags (optional, guides the generation)
    #[arg(long)]
    pub tags: Option<String>,

    /// Model version (default: config `default_model`, v5.5 out of the box)
    #[arg(short, long)]
    pub model: Option<ModelVersion>,

    /// Vocal gender
    #[arg(long)]
    pub vocal: Option<VocalGender>,

    /// Weirdness level (0-100)
    #[arg(long)]
    pub weirdness: Option<f64>,

    /// Style influence strength (0-100)
    #[arg(long)]
    pub style_influence: Option<f64>,

    /// Generate instrumental only
    #[arg(long)]
    pub instrumental: bool,

    /// Bypass the duplicate-run guard
    #[arg(long)]
    pub force: bool,

    /// Wait for generation to complete
    #[arg(short, long)]
    pub wait: bool,

    /// Download output to directory
    #[arg(long)]
    pub download: Option<String>,

    /// hCaptcha token (overrides the auto-solver)
    #[arg(long)]
    pub token: Option<String>,

    /// Skip the built-in hCaptcha auto-solver
    #[arg(long)]
    pub no_captcha: bool,

    /// Voice persona ID (generates with your custom voice)
    #[arg(long)]
    pub persona: Option<String>,
}

#[derive(clap::Args)]
pub struct LyricsArgs {
    /// What the song should be about
    #[arg(short, long)]
    pub prompt: String,
}

#[derive(clap::Args)]
pub struct ExtendArgs {
    /// Clip ID to extend
    pub clip_id: String,

    /// Timestamp in seconds to continue from
    #[arg(long)]
    pub at: f64,

    /// New lyrics for the extension
    #[arg(long)]
    pub lyrics: Option<String>,

    /// Style tags
    #[arg(long)]
    pub tags: Option<String>,

    /// Model version (default: config `default_model`, v5.5 out of the box)
    #[arg(short, long)]
    pub model: Option<ModelVersion>,

    /// hCaptcha token (overrides the auto-solver)
    #[arg(long)]
    pub token: Option<String>,

    /// Skip the built-in hCaptcha auto-solver
    #[arg(long)]
    pub no_captcha: bool,

    /// Wait for completion
    #[arg(short, long)]
    pub wait: bool,
}

#[derive(clap::Args)]
pub struct ConcatArgs {
    /// Clip ID to concatenate into a full song
    pub clip_id: String,

    /// Wait for completion
    #[arg(short, long)]
    pub wait: bool,
}

#[derive(clap::Args)]
pub struct CoverArgs {
    /// Clip ID to create a cover of
    pub clip_id: String,

    /// Style tags for the cover
    #[arg(long)]
    pub tags: Option<String>,

    /// Model version for the cover (default: config `default_model`)
    #[arg(short, long)]
    pub model: Option<ModelVersion>,

    /// Audio influence strength (0-100) — how strongly the source clip
    /// shapes the cover
    #[arg(long)]
    pub audio_influence: Option<f64>,

    /// Bypass the duplicate-run guard
    #[arg(long)]
    pub force: bool,

    /// hCaptcha token (overrides the auto-solver)
    #[arg(long)]
    pub token: Option<String>,

    /// Skip the built-in hCaptcha auto-solver
    #[arg(long)]
    pub no_captcha: bool,

    /// Wait for completion
    #[arg(short, long)]
    pub wait: bool,

    /// Download output to directory
    #[arg(long)]
    pub download: Option<String>,
}

#[derive(clap::Args)]
pub struct RemasterArgs {
    /// Clip ID to remaster
    pub clip_id: String,

    /// Remaster model version
    #[arg(long, default_value = "v5.5")]
    pub model: RemasterModel,

    /// Bypass the duplicate-run guard
    #[arg(long)]
    pub force: bool,

    /// hCaptcha token (overrides the auto-solver)
    #[arg(long)]
    pub token: Option<String>,

    /// Skip the built-in hCaptcha auto-solver
    #[arg(long)]
    pub no_captcha: bool,

    /// Wait for completion
    #[arg(short, long)]
    pub wait: bool,

    /// Download output to directory
    #[arg(long)]
    pub download: Option<String>,
}

#[derive(clap::Args)]
pub struct InfoArgs {
    /// Clip ID to inspect
    pub id: String,
}

#[derive(clap::Args)]
pub struct PersonaArgs {
    /// Persona ID to view
    pub id: String,
}

#[derive(clap::Args)]
pub struct StemsArgs {
    /// Clip ID to extract stems from
    pub clip_id: String,

    /// Wait for completion
    #[arg(short, long)]
    pub wait: bool,
}

#[derive(clap::Args)]
pub struct ListArgs {
    /// Opaque pagination cursor from a previous `list --json` response
    /// (`next_cursor`). Omit for the first page.
    #[arg(long)]
    pub cursor: Option<String>,
}

#[derive(clap::Args)]
pub struct SearchArgs {
    /// Search query (matches title and tags)
    pub query: String,
}

#[derive(clap::Args)]
pub struct DeleteArgs {
    /// Clip ID(s) to delete
    pub ids: Vec<String>,

    /// Skip confirmation
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(clap::Args)]
pub struct StatusArgs {
    /// Clip ID(s) to check
    pub ids: Vec<String>,
}

#[derive(clap::Args)]
pub struct DownloadArgs {
    /// Clip ID(s) to download
    pub ids: Vec<String>,

    /// Output directory (default: config `output_dir`, "." out of the box)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Download video instead of audio
    #[arg(long)]
    pub video: bool,
}

#[derive(clap::Args)]
pub struct SetArgs {
    /// Clip ID to update
    pub id: String,

    /// New title
    #[arg(long)]
    pub title: Option<String>,

    /// New lyrics text
    #[arg(long)]
    pub lyrics: Option<String>,

    /// Read lyrics from file
    #[arg(long)]
    pub lyrics_file: Option<String>,

    /// New caption
    #[arg(long)]
    pub caption: Option<String>,

    /// Remove custom cover image
    #[arg(long)]
    pub remove_cover: bool,
}

#[derive(clap::Args)]
pub struct PublishArgs {
    /// Clip ID(s)
    pub ids: Vec<String>,

    /// Make public (default) or --private
    #[arg(long)]
    pub private: bool,
}

#[derive(clap::Args)]
pub struct TimedLyricsArgs {
    /// Clip ID
    pub id: String,

    /// Output as LRC format
    #[arg(long)]
    pub lrc: bool,
}

#[derive(clap::Args)]
pub struct AuthArgs {
    /// Auto-extract from browser (recommended)
    #[arg(long)]
    pub login: bool,

    /// Force-refresh the JWT via the stored Clerk session cookie. Use this
    /// when the CLI returns `auth_expired` or `Token validation failed`
    /// without requiring a full re-login from the browser.
    #[arg(long)]
    pub refresh: bool,

    /// JWT token (manual fallback)
    #[arg(long)]
    pub jwt: Option<String>,

    /// Clerk __client cookie (manual fallback for headless servers)
    ///
    /// Accepts either the raw __client value or a full browser Cookie header.
    #[arg(long)]
    pub cookie: Option<String>,

    /// Device ID
    #[arg(long)]
    pub device: Option<String>,

    /// Remove stored authentication
    #[arg(long)]
    pub logout: bool,
}

#[derive(clap::Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(clap::Args)]
pub struct InstallSkillArgs {
    /// Custom output path (writes a single SKILL.md there instead of the
    /// per-platform install)
    #[arg(long)]
    pub path: Option<String>,

    /// Rewrite skill files even when already current
    #[arg(short, long)]
    pub force: bool,

    /// Print the skill content to stdout instead of writing
    #[arg(long)]
    pub print: bool,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show the effective merged configuration (defaults < file < SUNO_* env)
    Show,
    /// Set a configuration value in the config file
    Set { key: String, value: String },
    /// Show the configuration file path
    Path,
    /// Validate the configuration file
    Check,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum ModelVersion {
    #[value(name = "v5.5")]
    #[default]
    V55,
    #[value(name = "v5")]
    V5,
    #[value(name = "v4.5+")]
    V45Plus,
    #[value(name = "v4.5-all")]
    V45All,
    #[value(name = "v4.5")]
    V45,
    #[value(name = "v4")]
    V4,
    #[value(name = "v3.5")]
    V35,
    #[value(name = "v3")]
    V3,
    #[value(name = "v2")]
    V2,
}

impl ModelVersion {
    pub fn to_api_key(&self) -> &'static str {
        match self {
            Self::V55 => "chirp-fenix",
            Self::V5 => "chirp-crow",
            Self::V45Plus => "chirp-bluejay",
            Self::V45All => "chirp-auk-turbo",
            Self::V45 => "chirp-auk",
            Self::V4 => "chirp-v4",
            Self::V35 => "chirp-v3-5",
            Self::V3 => "chirp-v3-0",
            Self::V2 => "chirp-v2-xxl-alpha",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::V55 => "v5.5",
            Self::V5 => "v5",
            Self::V45Plus => "v4.5+",
            Self::V45All => "v4.5-all",
            Self::V45 => "v4.5",
            Self::V4 => "v4",
            Self::V35 => "v3.5",
            Self::V3 => "v3",
            Self::V2 => "v2",
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum VocalGender {
    Male,
    Female,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum RemasterModel {
    #[value(name = "v5.5")]
    #[default]
    V55,
    #[value(name = "v5")]
    V5,
    #[value(name = "v4.5+")]
    V45Plus,
}

impl RemasterModel {
    pub fn to_api_key(&self) -> &'static str {
        match self {
            Self::V55 => "chirp-flounder",
            Self::V5 => "chirp-carp",
            Self::V45Plus => "chirp-bass",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::ValueEnum;

    #[test]
    fn model_versions_map_to_api_keys() {
        // v4.5-all is the free-tier model missing from the enum until 0.6.0.
        assert_eq!(ModelVersion::V45All.to_api_key(), "chirp-auk-turbo");
        assert_eq!(ModelVersion::V55.to_api_key(), "chirp-fenix");

        // Every selectable --model value must have an API key and a display
        // name matching its clap value name (agent-info relies on this).
        for m in ModelVersion::value_variants() {
            assert!(m.to_api_key().starts_with("chirp"));
            let clap_name = m.to_possible_value().unwrap().get_name().to_string();
            assert_eq!(m.display_name(), clap_name);
        }
    }

    #[test]
    fn remaster_models_map_to_api_keys() {
        for m in RemasterModel::value_variants() {
            assert!(m.to_api_key().starts_with("chirp"));
        }
    }
}
