//! `suno write` — the native song composer. This is the integrated song
//! generator: it actively assembles a Suno-ready structured song (a Style
//! Prompt line, a meta-tagged section skeleton with lyric placeholders, and a
//! Suno Tags line) from a GENRE registry embedded here and derived faithfully
//! from the "Genre-specific tag combinations" section of
//! `assets/guides/songwriting.md`. The agent fills the `<...>` lyric
//! placeholders, then runs `suno generate --lyrics-file`.
//!
//! Modes are extensible: add a `WriteMode` variant in `cli.rs` and a branch in
//! `render_plain` / `compose` here — everything else derives from the registry.

use serde::Serialize;

use crate::cli::{VocalGender, WriteArgs, WriteMode};
use crate::errors::CliError;
use crate::output::OutputFormat;

/// One ready-made Style Prompt block, decomposed into the guide's assembly
/// formula: `{genre}, {mood}, {tempo} BPM, {vocal}, {instruments}`, plus the
/// meta-tags to front-load ([Instrument], [Mood], [Energy], [Vocal Style]).
pub struct Genre {
    pub key: &'static str,
    pub aliases: &'static [&'static str],
    /// Leading genre descriptor, e.g. "Modern pop".
    pub genre_tag: &'static str,
    /// Descriptive mood phrase for the Style Prompt, e.g. "catchy and polished".
    pub mood: &'static str,
    pub bpm: u32,
    /// Default vocal descriptor for the Style Prompt, e.g. "bright female vocals".
    pub vocal: &'static str,
    /// Instruments for the Style Prompt, e.g. "synths, punchy drums, bass drops".
    pub instruments: &'static str,
    /// The `[Instrument: ...]` meta-tag contents.
    pub instrument_tag: &'static str,
    /// The `[Mood: ...]` meta-tag word.
    pub mood_tag: &'static str,
    /// The `[Energy: ...]` meta-tag value.
    pub energy: &'static str,
    /// The `[Vocal Style: ...]` (or `[Texture: ...]`) meta-tag value.
    pub vocal_style: &'static str,
}

/// The embedded registry. Every entry is a faithful decomposition of a block
/// under "Genre-specific tag combinations" in the songwriting guide. Add a
/// genre = one entry; fuzzy match and the reassembled Style Prompt derive from
/// it. `chill-lounge` doubles as the priming-mode default.
pub const GENRES: &[Genre] = &[
    // --- Pop ---
    Genre {
        key: "modern-pop",
        aliases: &["pop", "modern pop", "dance-pop", "dance pop"],
        genre_tag: "Modern pop",
        mood: "catchy and polished",
        bpm: 120,
        vocal: "bright female vocals",
        instruments: "synths, punchy drums, bass drops",
        instrument_tag: "Synths, Programmed Drums, Bass",
        mood_tag: "Uplifting",
        energy: "High",
        vocal_style: "Bright, Clear",
    },
    Genre {
        key: "indie-pop",
        aliases: &["indie pop", "dream pop", "dreampop", "bedroom pop"],
        genre_tag: "Indie pop",
        mood: "dreamy and nostalgic",
        bpm: 100,
        vocal: "airy vocals",
        instruments: "acoustic guitar, soft synths",
        instrument_tag: "Acoustic Guitar, Soft Synths, Light Drums",
        mood_tag: "Nostalgic",
        energy: "Medium",
        vocal_style: "Airy, Intimate",
    },
    Genre {
        key: "synth-pop",
        aliases: &["synthpop", "synth pop", "80s", "1980s", "eighties"],
        genre_tag: "1980s synth-pop",
        mood: "retro and energetic",
        bpm: 118,
        vocal: "bright vocals with reverb",
        instruments: "analog synths, gated drums",
        instrument_tag: "Analog Synths, Gated Reverb Drums, Bass",
        mood_tag: "Energetic",
        energy: "High",
        vocal_style: "Retro, Bright",
    },
    // --- R&B / Soul ---
    Genre {
        key: "modern-rnb",
        aliases: &[
            "rnb",
            "rb",
            "r&b",
            "modern r&b",
            "modern rnb",
            "rhythm and blues",
        ],
        genre_tag: "Modern R&B",
        mood: "smooth and sensual",
        bpm: 85,
        vocal: "breathy vocals",
        instruments: "808s, soft keys, minimal production",
        instrument_tag: "808s, Rhodes, Soft Synths",
        mood_tag: "Romantic",
        energy: "Low→Medium",
        vocal_style: "Breathy, Smooth",
    },
    Genre {
        key: "neo-soul",
        aliases: &["neosoul", "neo soul", "soul"],
        genre_tag: "Neo-soul",
        mood: "warm and organic",
        bpm: 90,
        vocal: "rich vocals",
        instruments: "live drums, Rhodes piano, bass guitar",
        instrument_tag: "Rhodes, Live Drums, Upright Bass",
        mood_tag: "Warm",
        energy: "Medium",
        vocal_style: "Organic, Warm",
    },
    Genre {
        key: "90s-rnb",
        aliases: &[
            "90s r&b",
            "1990s r&b",
            "90s rnb",
            "r&b ballad",
            "rnb ballad",
        ],
        genre_tag: "1990s R&B ballad",
        mood: "romantic and smooth",
        bpm: 70,
        vocal: "silky vocals with harmonies",
        instruments: "piano, strings",
        instrument_tag: "Piano, Strings, Light Drums",
        mood_tag: "Romantic",
        energy: "Low",
        vocal_style: "Silky, Harmonies",
    },
    // --- Hip-Hop / Rap ---
    Genre {
        key: "trap",
        aliases: &["trap music"],
        genre_tag: "Trap",
        mood: "dark and hard-hitting",
        bpm: 140,
        vocal: "aggressive delivery",
        instruments: "heavy 808s, hi-hats, dark synths",
        instrument_tag: "Heavy 808s, Rapid Hi-hats, Dark Synths",
        mood_tag: "Intense",
        energy: "High",
        vocal_style: "Aggressive, Confident",
    },
    Genre {
        key: "boom-bap",
        aliases: &["boombap", "boom bap", "hip-hop", "hip hop", "hiphop", "rap"],
        genre_tag: "Boom bap hip-hop",
        mood: "classic and lyrical",
        bpm: 90,
        vocal: "confident flow",
        instruments: "dusty drums, jazz samples",
        instrument_tag: "Boom Bap Drums, Jazz Samples, Bass",
        mood_tag: "Confident",
        energy: "Medium",
        vocal_style: "Dusty, Classic",
    },
    Genre {
        key: "melodic-rap",
        aliases: &["melodic rap", "emo rap", "emo-rap"],
        genre_tag: "Melodic rap",
        mood: "emotional and atmospheric",
        bpm: 130,
        vocal: "auto-tuned vocals",
        instruments: "808s, ambient pads",
        instrument_tag: "808s, Ambient Synths, Hi-hats",
        mood_tag: "Emotional",
        energy: "Medium→High",
        vocal_style: "Melodic, Auto-tuned",
    },
    // --- Rock ---
    Genre {
        key: "alt-rock",
        aliases: &["alternative rock", "alt rock", "rock"],
        genre_tag: "Alternative rock",
        mood: "raw and emotional",
        bpm: 125,
        vocal: "powerful vocals",
        instruments: "distorted guitars, live drums",
        instrument_tag: "Electric Guitar, Bass, Live Drums",
        mood_tag: "Emotional",
        energy: "High",
        vocal_style: "Raw, Powerful",
    },
    Genre {
        key: "indie-rock",
        aliases: &["indie rock"],
        genre_tag: "Indie rock",
        mood: "jangly and nostalgic",
        bpm: 110,
        vocal: "warm vocals",
        instruments: "clean guitars, driving drums",
        instrument_tag: "Clean Electric Guitar, Bass, Drums",
        mood_tag: "Nostalgic",
        energy: "Medium→High",
        vocal_style: "Warm, Jangly",
    },
    Genre {
        key: "soft-rock",
        aliases: &["soft rock", "rock ballad", "ballad", "power ballad"],
        genre_tag: "Soft rock ballad",
        mood: "emotional and soaring",
        bpm: 75,
        vocal: "powerful vocals",
        instruments: "acoustic guitar, piano, strings",
        instrument_tag: "Acoustic Guitar, Piano, Strings, Light Drums",
        mood_tag: "Emotional",
        energy: "Low→High",
        vocal_style: "Powerful, Emotive",
    },
    // --- Electronic / Dance ---
    Genre {
        key: "house",
        aliases: &[
            "edm",
            "edm/house",
            "dance",
            "electronic",
            "four-on-the-floor",
        ],
        genre_tag: "House music",
        mood: "energetic and driving",
        bpm: 128,
        vocal: "catchy vocal hooks",
        instruments: "synths, four-on-the-floor",
        instrument_tag: "Synths, House Drums, Bass",
        mood_tag: "Euphoric",
        energy: "High",
        vocal_style: "Catchy, Bright",
    },
    Genre {
        key: "lo-fi",
        aliases: &["lofi", "lo fi", "chill", "chillhop", "lo-fi chill"],
        genre_tag: "Lo-fi chill",
        mood: "relaxed and hazy",
        bpm: 80,
        vocal: "soft distant vocals",
        instruments: "vinyl crackle, mellow keys",
        instrument_tag: "Mellow Keys, Lo-fi Drums, Vinyl Texture",
        mood_tag: "Relaxed",
        energy: "Low",
        vocal_style: "Lo-fi, Hazy, Warm",
    },
    Genre {
        key: "synthwave",
        aliases: &["retrowave", "outrun"],
        genre_tag: "Synthwave",
        mood: "nostalgic and cinematic",
        bpm: 100,
        vocal: "processed vocals",
        instruments: "retro synths, pulsing bass",
        instrument_tag: "Retro Synths, Pulsing Bass, Electronic Drums",
        mood_tag: "Nostalgic",
        energy: "Medium→High",
        vocal_style: "Cinematic, Retro",
    },
    // --- Country ---
    Genre {
        key: "country",
        aliases: &["modern country"],
        genre_tag: "Modern country",
        mood: "upbeat and feel-good",
        bpm: 120,
        vocal: "warm twangy vocals",
        instruments: "acoustic guitar, fiddle",
        instrument_tag: "Acoustic Guitar, Fiddle, Pedal Steel, Drums",
        mood_tag: "Feel-good",
        energy: "Medium→High",
        vocal_style: "Warm, Twangy",
    },
    Genre {
        key: "country-ballad",
        aliases: &["country ballad"],
        genre_tag: "Country ballad",
        mood: "emotional and heartfelt",
        bpm: 65,
        vocal: "sincere vocals",
        instruments: "acoustic guitar, steel guitar",
        instrument_tag: "Acoustic Guitar, Pedal Steel, Light Drums",
        mood_tag: "Heartfelt",
        energy: "Low",
        vocal_style: "Sincere, Emotive",
    },
    // --- Jazz / Blues ---
    Genre {
        key: "jazz",
        aliases: &["swing"],
        genre_tag: "Jazz",
        mood: "smooth and sophisticated",
        bpm: 100,
        vocal: "warm vocals",
        instruments: "piano trio, brushed drums",
        instrument_tag: "Piano, Upright Bass, Brushed Drums",
        mood_tag: "Sophisticated",
        energy: "Medium",
        vocal_style: "Warm, Jazzy Phrasing",
    },
    Genre {
        key: "blues",
        aliases: &["blues shuffle"],
        genre_tag: "Blues",
        mood: "raw and soulful",
        bpm: 65,
        vocal: "gritty vocals",
        instruments: "electric guitar, organ",
        instrument_tag: "Electric Guitar, Organ, Bass, Drums",
        mood_tag: "Soulful",
        energy: "Low→Medium",
        vocal_style: "Gritty, Soulful",
    },
    // --- Latin ---
    Genre {
        key: "reggaeton",
        aliases: &["dembow"],
        genre_tag: "Reggaeton",
        mood: "infectious and sultry",
        bpm: 95,
        vocal: "smooth Spanish vocals",
        instruments: "dembow beat, synth bass",
        instrument_tag: "Dembow Drums, Synth Bass, Percussion",
        mood_tag: "Sensual",
        energy: "High",
        vocal_style: "Smooth, Confident",
    },
    Genre {
        key: "latin-pop",
        aliases: &["latin pop", "latin"],
        genre_tag: "Latin pop",
        mood: "romantic and upbeat",
        bpm: 110,
        vocal: "passionate vocals",
        instruments: "acoustic guitar, percussion",
        instrument_tag: "Acoustic Guitar, Congas, Piano, Strings",
        mood_tag: "Romantic",
        energy: "Medium→High",
        vocal_style: "Passionate, Warm",
    },
    // --- Folk / Acoustic ---
    Genre {
        key: "folk",
        aliases: &["acoustic"],
        genre_tag: "Folk",
        mood: "intimate and storytelling",
        bpm: 95,
        vocal: "warm natural vocals",
        instruments: "acoustic guitar, light percussion",
        instrument_tag: "Acoustic Guitar, Mandolin, Light Percussion",
        mood_tag: "Intimate",
        energy: "Low→Medium",
        vocal_style: "Natural, Storytelling",
    },
    Genre {
        key: "singer-songwriter",
        aliases: &["singer songwriter", "songwriter"],
        genre_tag: "Singer-songwriter",
        mood: "vulnerable and honest",
        bpm: 70,
        vocal: "intimate vocals",
        instruments: "fingerpicked guitar, piano",
        instrument_tag: "Fingerpicked Guitar, Piano",
        mood_tag: "Vulnerable",
        energy: "Low",
        vocal_style: "Intimate, Honest",
    },
    // --- Gospel / Spiritual ---
    Genre {
        key: "gospel",
        aliases: &["spiritual"],
        genre_tag: "Gospel",
        mood: "uplifting and powerful",
        bpm: 100,
        vocal: "soaring vocals with choir",
        instruments: "organ, claps",
        instrument_tag: "Organ, Gospel Choir, Handclaps, Drums",
        mood_tag: "Uplifting",
        energy: "High",
        vocal_style: "Powerful, Soulful",
    },
    Genre {
        key: "worship",
        aliases: &["contemporary worship", "christian"],
        genre_tag: "Contemporary worship",
        mood: "inspiring and reverent",
        bpm: 75,
        vocal: "sincere vocals",
        instruments: "atmospheric pads, acoustic guitar",
        instrument_tag: "Acoustic Guitar, Ambient Pads, Light Drums",
        mood_tag: "Reverent",
        energy: "Medium",
        vocal_style: "Sincere, Warm",
    },
    // --- Priming default (chill lounge, ~72 BPM, low-arousal) ---
    Genre {
        key: "chill-lounge",
        aliases: &["lounge", "chill lounge", "downtempo"],
        genre_tag: "chill lounge",
        mood: "downtempo, low-arousal, relaxed-luxury",
        bpm: 72,
        vocal: "intimate vocal",
        instruments: "warm Rhodes, soft brushed drums",
        instrument_tag: "Warm Rhodes, Soft Brushed Drums, Upright Bass",
        mood_tag: "Relaxed",
        energy: "Low",
        vocal_style: "Intimate, Warm, Close-mic",
    },
];

/// Structure-tag vocabulary embedded for reference (agent-info / docs can
/// surface it; the skeleton below front-loads a curated subset).
pub const STRUCTURE_TAGS: &[&str] = &[
    "[Intro]",
    "[Verse]",
    "[Pre-Chorus]",
    "[Chorus]",
    "[Bridge]",
    "[Outro]",
    "[Fade Out]",
    "[Catchy Hook]",
    "[Instrumental]",
];

/// Normalize a genre string for fuzzy matching: lowercase, drop everything
/// that is not ASCII alphanumeric (spaces, hyphens, slashes, ampersands).
fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Resolve a genre string to a registry entry. Case-insensitive and fuzzy:
/// exact key/alias first, then a normalized contains-match either direction.
/// Returns `None` for an unknown genre — the caller uses the raw string as the
/// style tag so `write` never fails on genre.
pub fn resolve_genre(input: &str) -> Option<&'static Genre> {
    let norm = normalize(input);
    if norm.is_empty() {
        return None;
    }
    // Exact key or alias.
    if let Some(g) = GENRES
        .iter()
        .find(|g| normalize(g.key) == norm || g.aliases.iter().any(|a| normalize(a) == norm))
    {
        return Some(g);
    }
    // Fuzzy contains: input contains a key/alias, or a key/alias contains input
    // (min length 3 to avoid spurious two-letter hits). First match in array
    // order wins, so the registry order is the tie-break priority.
    GENRES.iter().find(|g| {
        let cands = std::iter::once(g.key).chain(g.aliases.iter().copied());
        cands.filter(|c| normalize(c).len() >= 3).any(|c| {
            let cn = normalize(c);
            norm.contains(&cn) || cn.contains(&norm)
        })
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Gender {
    Male,
    Female,
}

impl Gender {
    fn word(self) -> &'static str {
        match self {
            Gender::Male => "male",
            Gender::Female => "female",
        }
    }
    fn tag(self) -> &'static str {
        match self {
            Gender::Male => "[Male Vocal]",
            Gender::Female => "[Female Vocal]",
        }
    }
}

/// Resolve the vocal gender: an explicit `--vocal` flag wins, else infer from
/// the genre's default descriptor ("female" before "male" — the former
/// contains the latter as a substring), else `None`.
fn resolve_gender(flag: Option<&VocalGender>, vocal_desc: &str) -> Option<Gender> {
    if let Some(g) = flag {
        return Some(match g {
            VocalGender::Male => Gender::Male,
            VocalGender::Female => Gender::Female,
        });
    }
    let low = vocal_desc.to_lowercase();
    if low.contains("female") {
        Some(Gender::Female)
    } else if low.contains("male") {
        Some(Gender::Male)
    } else {
        None
    }
}

/// Fold a resolved gender into the vocal descriptor. Registry descriptors are
/// lowercase, so plain substitution is safe.
fn apply_vocal_gender(desc: &str, gender: Option<Gender>) -> String {
    let Some(g) = gender else {
        return desc.to_string();
    };
    let gw = g.word();
    let low = desc.to_lowercase();
    if low.contains("female") {
        desc.replace("female", gw)
    } else if low.contains("male") {
        desc.replace("male", gw)
    } else if desc.contains("vocals") {
        desc.replacen("vocals", &format!("{gw} vocals"), 1)
    } else if desc.contains("vocal") {
        desc.replacen("vocal", &format!("{gw} vocal"), 1)
    } else {
        format!("{desc}, {gw} vocal")
    }
}

/// A contrasting mood word for the bridge (dopamine architecture: contrast is
/// the point). Uplifting moods drop to reflective; darker moods lift.
fn contrast_mood(mood_tag: &str) -> &'static str {
    match mood_tag {
        "Melancholic" | "Reflective" | "Vulnerable" | "Heartfelt" | "Soulful" | "Emotional" => {
            "Uplifting"
        }
        _ => "Reflective",
    }
}

/// Title-case a short phrase for a fallback title.
fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut ch = w.chars();
            match ch.next() {
                Some(f) => f.to_uppercase().collect::<String>() + ch.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The fully-resolved song, ready to render as plain text or a JSON envelope.
struct Composition {
    title: String,
    mode: WriteMode,
    genre_key: String,
    genre_tag: String,
    mood: String,
    mood_tag: String,
    bpm: u32,
    vocal: String,
    instruments: String,
    instrument_tag: String,
    energy: String,
    vocal_style: String,
    gender: Option<Gender>,
    theme: Option<String>,
    viral: bool,
    instrumental: bool,
    priming: Option<PrimingMeta>,
}

struct PrimingMeta {
    target: String,
    objective: String,
    domain: String,
    subtlety: String,
}

/// Build a `Composition` from parsed args. Never errors on genre: an unknown
/// genre becomes a raw style tag with generic defaults.
fn compose(args: &WriteArgs) -> Composition {
    // Priming mode defaults to the chill-lounge scaffold when no genre is set.
    let default_genre = match args.mode {
        WriteMode::Priming => Some("chill-lounge"),
        WriteMode::Songwriting => None,
    };
    let genre_input = args.genre.as_deref().or(default_genre);

    let matched = genre_input.and_then(resolve_genre);

    let (
        genre_key,
        genre_tag,
        mut mood,
        def_bpm,
        def_vocal,
        instruments,
        instrument_tag,
        mood_tag,
        energy,
        vocal_style,
    ) = match matched {
        Some(g) => (
            g.key.to_string(),
            g.genre_tag.to_string(),
            g.mood.to_string(),
            g.bpm,
            g.vocal.to_string(),
            g.instruments.to_string(),
            g.instrument_tag.to_string(),
            g.mood_tag.to_string(),
            g.energy.to_string(),
            g.vocal_style.to_string(),
        ),
        None => {
            // Unknown genre: use the raw string verbatim as the style tag,
            // with genre-neutral defaults so the scaffold is still complete.
            let raw = genre_input.unwrap_or("pop").to_string();
            (
                raw.clone(),
                raw,
                "expressive".to_string(),
                110,
                "lead vocals".to_string(),
                "guitar, bass, drums".to_string(),
                "Guitar, Bass, Drums".to_string(),
                "Uplifting".to_string(),
                "Medium→High".to_string(),
                "Clear, Expressive".to_string(),
            )
        }
    };

    // --mood overrides the descriptive mood phrase in the Style Prompt.
    if let Some(m) = args.mood.as_deref() {
        mood = m.to_string();
    }

    let bpm = args.bpm.unwrap_or(def_bpm);
    let gender = if args.instrumental {
        None
    } else {
        resolve_gender(args.vocal.as_ref(), &def_vocal)
    };
    let vocal = apply_vocal_gender(&def_vocal, gender);

    let title = args
        .title
        .clone()
        .or_else(|| args.theme.as_deref().map(title_case))
        .unwrap_or_else(|| match args.mode {
            WriteMode::Priming => "Untitled Session".to_string(),
            WriteMode::Songwriting => "Untitled Draft".to_string(),
        });

    let priming = match args.mode {
        WriteMode::Priming => Some(PrimingMeta {
            target: args.target.clone().unwrap_or_else(|| {
                "<named consenting target OR anonymised batch descriptor>".into()
            }),
            objective: args
                .objective
                .clone()
                .unwrap_or_else(|| "<specific, falsifiable attitude or action>".into()),
            domain: args.domain.clone().unwrap_or_else(|| "other".into()),
            subtlety: args.subtlety.clone().unwrap_or_else(|| "medium".into()),
        }),
        WriteMode::Songwriting => None,
    };

    Composition {
        title,
        mode: args.mode,
        genre_key,
        genre_tag,
        mood,
        mood_tag,
        bpm,
        vocal,
        instruments,
        instrument_tag,
        energy,
        vocal_style,
        gender,
        theme: args.theme.clone(),
        viral: args.viral,
        instrumental: args.instrumental,
        priming,
    }
}

impl Composition {
    /// The assembled "Style Prompt" line: `{genre}, {mood}, {bpm} BPM,
    /// {vocal|instrumental}, {instruments}` (+ earworm tags when viral).
    fn style_prompt(&self) -> String {
        let mut parts = vec![
            self.genre_tag.clone(),
            self.mood.clone(),
            format!("{} BPM", self.bpm),
        ];
        if self.instrumental {
            parts.push("instrumental".to_string());
        } else {
            parts.push(self.vocal.clone());
        }
        parts.push(self.instruments.clone());
        if self.viral {
            parts.push("catchy hook".to_string());
            parts.push("earworm".to_string());
        }
        parts.join(", ")
    }

    /// The lower-level "Suno Tags" comma list: genre, instruments, vocal style,
    /// mood/energy, plus viral/instrumental extras. Deduplicated, order-stable.
    fn suno_tags(&self) -> String {
        let mut tags: Vec<String> = Vec::new();
        let push = |t: &str, tags: &mut Vec<String>| {
            let t = t.trim();
            if !t.is_empty() && !tags.iter().any(|x| x.eq_ignore_ascii_case(t)) {
                tags.push(t.to_string());
            }
        };
        push(&self.genre_tag, &mut tags);
        push(&self.mood_tag.to_lowercase(), &mut tags);
        for i in self.instruments.split(',') {
            push(i, &mut tags);
        }
        for v in self.vocal_style.split(',') {
            push(&v.to_lowercase(), &mut tags);
        }
        if self.instrumental {
            push("instrumental", &mut tags);
        }
        if self.viral {
            for t in [
                "catchy hook",
                "earworm",
                "singalong",
                "anthem",
                "gang vocals",
                "millennial whoop",
            ] {
                push(t, &mut tags);
            }
        }
        tags.join(", ")
    }

    /// The meta-tagged section skeleton with inline `<...>` placeholders. This
    /// is the block the agent fills and hands to `suno generate --lyrics-file`.
    fn structure(&self) -> String {
        let theme = self.theme.as_deref().unwrap_or("your theme");
        let hook = if self.viral { "[Catchy Hook]\n" } else { "" };
        let contrast = contrast_mood(&self.mood_tag);
        let mut s = String::new();

        // Intro / front-loaded meta tags.
        s.push_str(&format!(
            "[Intro] [Mood: {}] [Energy: {}]\n[Instrument: {}]\n",
            self.mood_tag, self.energy, self.instrument_tag
        ));
        if self.instrumental {
            s.push_str("[Instrumental]\n");
        } else {
            match self.gender {
                Some(g) => s.push_str(&format!("{}\n", g.tag())),
                None => s.push_str("[Vocal: <male or female>]\n"),
            }
            s.push_str(&format!("[Vocal Style: {}]\n", self.vocal_style));
        }
        if self.viral {
            s.push_str("<land the hook in the first 7 seconds — the streaming skip threshold>\n");
        }

        // Each section carries a vocal placeholder and a distinct instrumental
        // direction; instrumental mode swaps in the latter so the scaffold never
        // asks for "lines" in a song with no vocals.
        let body = |vocal_hint: &str, instr_hint: &str| -> String {
            if self.instrumental {
                format!("<instrumental — {instr_hint}: {}>", self.instruments)
            } else {
                vocal_hint.to_string()
            }
        };

        let chorus_vocal = if self.viral {
            "<the hook: 3-5 words, open vowels (oh/ah/ee/oo), repeat verbatim every chorus>"
        } else {
            "<4-6 lines — the memorable hook; repeat verbatim every chorus>"
        };

        s.push_str(&format!(
            "\n[Verse 1]\n{}\n",
            body(
                &format!("<4-6 lines, 6-10 syllables each — set the scene: {theme}>"),
                "state the main motif, set the mood",
            )
        ));
        s.push_str(&format!(
            "\n[Pre-Chorus] [Energy: Medium→High]\n{}\n",
            body(
                "<2-4 lines — lift the energy toward the hook>",
                "build tension toward the drop",
            )
        ));
        s.push_str(&format!(
            "\n{hook}[Chorus] [Energy: High]\n{}\n",
            body(
                chorus_vocal,
                "the main hook — lead line, fullest arrangement"
            )
        ));
        s.push_str(&format!(
            "\n[Verse 2]\n{}\n",
            body(
                &format!("<4-6 lines — develop the story: {theme}>"),
                "develop the motif, add a counter-line",
            )
        ));
        s.push_str(&format!(
            "\n[Bridge] [Mood: {contrast}] [Energy: Low→High]\n{}\n",
            body(
                "<2-4 lines — contrast; strip it back, then build to the last chorus>",
                "breakdown, then build back to the hook",
            )
        ));
        s.push_str(&format!(
            "\n{hook}[Chorus] [Energy: High]\n{}\n",
            body("<repeat the chorus verbatim>", "restate the hook")
        ));
        s.push_str(&format!(
            "\n[Outro] [Fade Out]\n{}\n",
            body(
                "<1-2 closing lines or let the instrumentation ride out>",
                "resolve and fade out",
            )
        ));

        s
    }

    /// The `suno generate` command a caller runs after filling the lyrics.
    fn generate_hint(&self) -> String {
        format!(
            "suno generate --title \"{}\" --tags \"{}\" --lyrics-file song.txt --model v4.5-all --wait --download ./",
            self.title,
            self.style_prompt()
        )
    }

    /// The priming Prime-Stack Map template + research-artefact skeleton. Kept
    /// deliberately light — the heavy methodology lives in `suno guide priming`.
    fn prime_stack_block(&self) -> String {
        let Some(p) = self.priming.as_ref() else {
            return String::new();
        };
        format!(
            "Prime-Stack Map\n\
             Select evidence-graded primes via `suno guide priming` (Grade B+ for load-bearing; Grade C decoration only).\n\
             | Section | Prime | Evidence Grade | Rationale |\n\
             |---|---|---|---|\n\
             | [Verse 1] | <seed prime — e.g. mere exposure> | <A/B> | <first exposure; brand/name in the first 10s> |\n\
             | [Chorus]  | <load-bearing prime — rhyme-as-reason> | <A/B> | <claim on a rhymed line; repeat verbatim> |\n\
             | [Bridge]  | <peak prime — peak-end rule> | <A> | <densest stacking here> |\n\
             | [Outro]   | <recency prime> | <A> | <last 10s carry the strongest memory trace> |\n\
             \n\
             Research Artefact\n\
             Target: {target}\n\
             Objective: {objective}\n\
             Domain: {domain}\n\
             Subtlety: {subtlety}\n\
             Predicted effect (calibrated priors): recall d≈0.4-0.6 · positive affect d≈0.3-0.5 · attitude shift d≈0.1-0.3 · behaviour d≈0.0-0.15\n\
             Ordering rule: never claim behaviour > attitude > recall.\n\
             Consent frame, prime library, phonetic name-embedding, and pre-registration template: `suno guide priming`.",
            target = p.target,
            objective = p.objective,
            domain = p.domain,
            subtlety = p.subtlety,
        )
    }

    /// Full plain-text rendering (Suno-ready song scaffold + priming artefact).
    fn render_plain(&self) -> String {
        let mut out = format!(
            "{title}\n\nStyle Prompt\n{style}\n\n---\n\nLyrics\n\n{structure}\n---\n\nSuno Tags\n{tags}\n",
            title = self.title,
            style = self.style_prompt(),
            structure = self.structure(),
            tags = self.suno_tags(),
        );
        if self.mode == WriteMode::Priming {
            out.push_str("\n---\n\n");
            out.push_str(&self.prime_stack_block());
            out.push('\n');
        }
        out
    }

    /// JSON envelope data: {title, mode, style_prompt, structure, suno_tags, ...}.
    fn to_json(&self) -> serde_json::Value {
        let mut v = serde_json::json!({
            "title": self.title,
            "mode": self.mode.as_str(),
            "genre": self.genre_key,
            "style_prompt": self.style_prompt(),
            "structure": self.structure(),
            "suno_tags": self.suno_tags(),
            "structure_tags": STRUCTURE_TAGS,
            "bpm": self.bpm,
            "vocal": self.gender.map(|g| g.word()),
            "theme": self.theme,
            "viral": self.viral,
            "instrumental": self.instrumental,
            "generate": self.generate_hint(),
        });
        if let Some(p) = self.priming.as_ref() {
            v["priming"] = serde_json::json!({
                "target": p.target,
                "objective": p.objective,
                "domain": p.domain,
                "subtlety": p.subtlety,
                "prime_stack_map": self.prime_stack_block(),
            });
        }
        v
    }
}

/// A serializable wrapper for the JSON envelope with a stable `written` field.
#[derive(Serialize)]
struct WriteEnvelope {
    #[serde(flatten)]
    song: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    written: Option<String>,
}

pub fn run(args: WriteArgs, fmt: OutputFormat, quiet: bool) -> Result<(), CliError> {
    let comp = compose(&args);
    let plain = comp.render_plain();

    // --out writes the plain-text song to a file regardless of output format.
    if let Some(path) = args.out.as_deref() {
        std::fs::write(path, &plain).map_err(|e| {
            CliError::Io(std::io::Error::new(
                e.kind(),
                format!("cannot write '{path}': {e}"),
            ))
        })?;
    }

    match fmt {
        OutputFormat::Json => {
            let envelope = WriteEnvelope {
                song: comp.to_json(),
                written: args.out.clone(),
            };
            crate::output::json::success(&envelope);
        }
        OutputFormat::Table => {
            if let Some(path) = args.out.as_deref() {
                if !quiet {
                    eprintln!("Wrote song scaffold to {path}");
                }
            } else {
                // Raw plain text to stdout, like `timed-lyrics --lrc`.
                print!("{plain}");
            }
            if !quiet {
                eprintln!("\n## write → generate");
                eprintln!(
                    "Fill the <...> lyric placeholders (save the Lyrics block to song.txt), then:"
                );
                eprintln!("  {}", comp.generate_hint());
                if comp.mode == WriteMode::Priming {
                    eprintln!(
                        "Priming: complete the Prime-Stack Map with graded primes — `suno guide priming`."
                    );
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_args() -> WriteArgs {
        WriteArgs {
            theme: None,
            genre: None,
            mood: None,
            vocal: None,
            bpm: None,
            viral: false,
            instrumental: false,
            title: None,
            mode: WriteMode::Songwriting,
            target: None,
            objective: None,
            domain: None,
            subtlety: None,
            out: None,
        }
    }

    #[test]
    fn genre_lookup_known_and_fuzzy() {
        assert_eq!(resolve_genre("indie-rock").unwrap().key, "indie-rock");
        // Case-insensitive alias.
        assert_eq!(resolve_genre("Indie Rock").unwrap().key, "indie-rock");
        // Alias hit.
        assert_eq!(resolve_genre("pop").unwrap().key, "modern-pop");
        // Fuzzy contains.
        assert_eq!(resolve_genre("trap music vibes").unwrap().key, "trap");
        // R&B punctuation normalizes.
        assert_eq!(resolve_genre("R&B").unwrap().key, "modern-rnb");
    }

    #[test]
    fn genre_lookup_unknown_passes_through() {
        assert!(resolve_genre("qwertypunk").is_none());
        // compose() uses the raw string verbatim as the genre/style tag.
        let mut args = base_args();
        args.genre = Some("Balkan brass funk".to_string());
        let comp = compose(&args);
        assert_eq!(comp.genre_key, "Balkan brass funk");
        assert!(comp.style_prompt().contains("Balkan brass funk"));
    }

    #[test]
    fn every_genre_key_and_alias_resolves() {
        for g in GENRES {
            assert_eq!(resolve_genre(g.key).map(|x| x.key), Some(g.key));
            for a in g.aliases {
                assert!(resolve_genre(a).is_some(), "alias '{a}' did not resolve");
            }
        }
        // Cover all ~24 subgenres plus the chill-lounge priming default.
        assert!(GENRES.len() >= 24);
    }

    #[test]
    fn songwriting_mode_has_anchors() {
        let comp = compose(&base_args());
        let out = comp.render_plain();
        assert!(!out.trim().is_empty());
        assert!(out.contains("Style Prompt"));
        assert!(out.contains("[Chorus]"));
        assert!(out.contains("[Verse 1]"));
        assert!(out.contains("[Bridge]"));
        assert!(out.contains("Suno Tags"));
    }

    #[test]
    fn priming_mode_defaults_to_chill_lounge() {
        let mut args = base_args();
        args.mode = WriteMode::Priming;
        let comp = compose(&args);
        let out = comp.render_plain();
        assert!(out.contains("Prime-Stack"));
        assert!(out.contains("chill"));
        assert!(out.contains("72"));
        assert_eq!(comp.bpm, 72);
        // The song scaffold is still present.
        assert!(out.contains("[Chorus]"));
        assert!(out.contains("Style Prompt"));
    }

    #[test]
    fn priming_carries_flags() {
        let mut args = base_args();
        args.mode = WriteMode::Priming;
        args.domain = Some("investment".into());
        args.subtlety = Some("stealth".into());
        args.target = Some("batch: seed investors".into());
        let comp = compose(&args);
        let out = comp.render_plain();
        assert!(out.contains("investment"));
        assert!(out.contains("stealth"));
        assert!(out.contains("batch: seed investors"));
    }

    #[test]
    fn viral_adds_hook_tags() {
        let mut args = base_args();
        args.genre = Some("pop".into());
        args.viral = true;
        let comp = compose(&args);
        assert!(comp.style_prompt().contains("earworm"));
        assert!(comp.style_prompt().contains("catchy hook"));
        let tags = comp.suno_tags();
        assert!(tags.contains("singalong"));
        assert!(tags.contains("gang vocals"));
        assert!(comp.structure().contains("[Catchy Hook]"));
    }

    #[test]
    fn json_envelope_shape() {
        let mut args = base_args();
        args.genre = Some("indie-rock".into());
        args.theme = Some("late nights".into());
        let v = compose(&args).to_json();
        assert_eq!(v["mode"], "songwriting");
        assert_eq!(v["genre"], "indie-rock");
        assert!(v["style_prompt"].as_str().unwrap().contains("Indie rock"));
        assert!(v["structure"].as_str().unwrap().contains("[Chorus]"));
        assert!(!v["suno_tags"].as_str().unwrap().is_empty());
        assert!(v["generate"].as_str().unwrap().contains("suno generate"));
        assert_eq!(v["title"], "Late Nights");
    }

    #[test]
    fn priming_json_has_priming_block() {
        let mut args = base_args();
        args.mode = WriteMode::Priming;
        args.domain = Some("marketing".into());
        let v = compose(&args).to_json();
        assert_eq!(v["mode"], "priming");
        assert_eq!(v["priming"]["domain"], "marketing");
        assert!(
            v["priming"]["prime_stack_map"]
                .as_str()
                .unwrap()
                .contains("Prime-Stack Map")
        );
    }

    #[test]
    fn vocal_flag_overrides_gender() {
        let mut args = base_args();
        args.genre = Some("modern-pop".into()); // default female
        args.vocal = Some(VocalGender::Male);
        let comp = compose(&args);
        assert_eq!(comp.gender, Some(Gender::Male));
        assert!(comp.vocal.contains("male"));
        assert!(!comp.vocal.contains("female"));
        assert!(comp.structure().contains("[Male Vocal]"));
    }

    #[test]
    fn instrumental_drops_vocals() {
        let mut args = base_args();
        args.genre = Some("lo-fi".into());
        args.instrumental = true;
        let comp = compose(&args);
        assert!(comp.style_prompt().contains("instrumental"));
        let structure = comp.structure();
        assert!(structure.contains("[Instrumental]"));
        assert!(!structure.contains("[Vocal Style"));
        assert!(comp.suno_tags().contains("instrumental"));
    }

    #[test]
    fn structure_tags_cover_the_skeleton() {
        // The embedded vocabulary must include every tag the skeleton emits.
        let mut args = base_args();
        args.viral = true;
        let structure = compose(&args).structure();
        for tag in [
            "[Intro]",
            "[Chorus]",
            "[Bridge]",
            "[Outro]",
            "[Catchy Hook]",
        ] {
            assert!(
                STRUCTURE_TAGS.contains(&tag),
                "{tag} missing from vocabulary"
            );
            assert!(structure.contains(tag), "{tag} missing from skeleton");
        }
    }

    #[test]
    fn bpm_and_mood_overrides_apply() {
        let mut args = base_args();
        args.genre = Some("jazz".into());
        args.bpm = Some(88);
        args.mood = Some("brooding and noir".into());
        let comp = compose(&args);
        let sp = comp.style_prompt();
        assert!(sp.contains("88 BPM"));
        assert!(sp.contains("brooding and noir"));
    }
}
