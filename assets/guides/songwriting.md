# Songwriting for Suno

Run `suno write` to scaffold the song; use this guide to fill and refine its output. `suno write --out song.txt` assembles the Style Prompt, the meta-tagged section skeleton, and the Suno Tags from the same grammar written up here, and prints the exact `suno generate` command — this document is the reference for what goes in the `<...>` slots and why, and for anything the scaffold does not cover.

This is the base grammar of writing for Suno: the structure, meta tags, syllable discipline, style formula, and vocal/hook vocabulary that produce a clean, generatable song. Every other guide (`suno guide priming`, and any genre- or effect-specific guide) assumes what is written here and extends it.

The song you write here is plain text that pastes straight into Suno or feeds the `suno` CLI. This document is markdown; the actual lyric/style output is not — see the output format below.

## Workflow

0. Run `suno write --genre <genre> --theme <theme> --out song.txt`. Steps 1-5 below are then a refinement pass over what it produced; write from scratch only if the scaffold does not fit.
1. Collect from the user: theme, genre, mood, vocal style. Supply sensible defaults for anything missing — do not block on questions.
2. Pick genre-specific tag combinations from the genre tables below.
3. Choose vocal direction, effects, and phrasing from the vocal styles section.
4. If the user wants catchy / viral / earworm / TikTok-ready, apply the viral hook techniques.
5. Write the lyrics using the output format below.
6. Generate audio with the `suno` CLI (see **Generate it** at the end).

## Output format

Plain text only. NEVER use markdown formatting in the song itself — no `**`, no `#`, no backticks. The output must copy-paste directly into Suno.

```
{Title}

Style Prompt
{genre}, {mood}, {tempo} BPM, {vocal type}, {instruments}

---

Lyrics

[Intro] [Mood: {mood}] [Energy: {level}]
[Instrument: {instruments}]
[Vocal Style: {style}]

[Verse 1]
{4-6 lines, 6-10 syllables each}

[Pre-Chorus]
{2-4 lines}

[Chorus] [Energy: High]
{4-6 lines, memorable hook}

[Verse 2]
{4-6 lines}

[Bridge] [Mood: {contrast}]
{2-4 lines}

[Chorus] [Energy: High]
{repeat verbatim}

[Outro] [Fade Out]

---

Suno Tags
{comma-separated list}
```

## Rules

- **Plain text only** — no markdown, no commentary, no song notes in the output.
- **Lines**: 6-10 syllables, 4-6 words. Anything longer gets crushed or rushed by the model.
- **Tags**: Front-load structural/mood tags in the first 3-5 lines so the model locks the direction early.
- **Style formula**: 1-2 genres + 1 mood + 2-3 instruments max. Overloading the style prompt muddies the result.
- **Chorus**: Repeat it verbatim every time for consistency. Do not paraphrase across choruses.
- **Punctuation**: Commas = pause. Ellipses = breath. Hyphens = connected syllables. Line breaks = natural breathing points.
- **No real artist names** in Style Prompts or Suno Tags. Describe the sound qualities instead:
  - "Adele style" → "powerful female vocal, dramatic ballad, soulful belt, wide dynamic range"
  - "Billie Eilish" → "breathy whisper vocal, intimate, dark pop, minimalist"
  - You CAN mention genres, decades, film references, and technical vocal terms.

## Structure tags

| Category | Tags |
|----------|------|
| Structure | `[Intro]` `[Verse]` `[Pre-Chorus]` `[Chorus]` `[Bridge]` `[Outro]` `[Fade Out]` |
| Energy | `[Energy: Low]` `[Energy: Medium]` `[Energy: High]` `[Energy: Medium→High]` |
| Mood | `[Mood: Uplifting]` `[Mood: Romantic]` `[Mood: Melancholic]` `[Mood: Intense]` |
| Vocals | `[Vocal Style: Breathy]` `[Vocal Style: Powerful]` `[Female Vocal]` `[Male Vocal]` |

## Genre-specific tag combinations

Each block below is a ready-made Style Prompt line plus the meta tags to front-load. Pick one, adjust the theme, keep the shape.

### Pop

**Modern Pop**
```
Modern pop, catchy and polished, 120 BPM, bright female vocals, synths, punchy drums, bass drops
[Instrument: Synths, Programmed Drums, Bass]
[Mood: Uplifting] [Energy: High]
[Vocal Style: Bright, Clear]
```

**Indie Pop**
```
Indie pop, dreamy and nostalgic, mid-tempo 100 BPM, airy vocals, acoustic guitar, soft synths
[Instrument: Acoustic Guitar, Soft Synths, Light Drums]
[Mood: Nostalgic] [Energy: Medium]
[Vocal Style: Airy, Intimate]
```

**Synth Pop / 80s**
```
1980s synth-pop, retro and energetic, 118 BPM, bright vocals with reverb, analog synths, gated drums
[Instrument: Analog Synths, Gated Reverb Drums, Bass]
[Mood: Energetic] [Energy: High]
[Texture: Retro, Bright]
```

### R&B / Soul

**Modern R&B**
```
Modern R&B, smooth and sensual, 85 BPM, breathy vocals, 808s, soft keys, minimal production
[Instrument: 808s, Rhodes, Soft Synths]
[Mood: Romantic] [Energy: Low→Medium]
[Vocal Style: Breathy, Smooth]
```

**Neo-Soul**
```
Neo-soul, warm and organic, 90 BPM, rich vocals, live drums, Rhodes piano, bass guitar
[Instrument: Rhodes, Live Drums, Upright Bass]
[Mood: Warm] [Energy: Medium]
[Texture: Organic, Warm]
```

**90s R&B**
```
1990s R&B ballad, romantic and smooth, slow 70 BPM, silky vocals with harmonies, piano, strings
[Instrument: Piano, Strings, Light Drums]
[Mood: Romantic] [Energy: Low]
[Vocal Style: Silky, Harmonies]
```

### Hip-Hop / Rap

**Trap**
```
Trap, dark and hard-hitting, 140 BPM, aggressive delivery, heavy 808s, hi-hats, dark synths
[Instrument: Heavy 808s, Rapid Hi-hats, Dark Synths]
[Mood: Intense] [Energy: High]
[Vocal Style: Aggressive, Confident]
```

**Boom Bap**
```
Boom bap hip-hop, classic and lyrical, 90 BPM, confident flow, dusty drums, jazz samples
[Instrument: Boom Bap Drums, Jazz Samples, Bass]
[Mood: Confident] [Energy: Medium]
[Texture: Dusty, Classic]
```

**Melodic Rap**
```
Melodic rap, emotional and atmospheric, 130 BPM, auto-tuned vocals, 808s, ambient pads
[Instrument: 808s, Ambient Synths, Hi-hats]
[Mood: Emotional] [Energy: Medium→High]
[Vocal Style: Melodic, Auto-tuned]
```

### Rock

**Alternative Rock**
```
Alternative rock, raw and emotional, 125 BPM, powerful vocals, distorted guitars, live drums
[Instrument: Electric Guitar, Bass, Live Drums]
[Mood: Emotional] [Energy: High]
[Vocal Style: Raw, Powerful]
```

**Indie Rock**
```
Indie rock, jangly and nostalgic, 110 BPM, warm vocals, clean guitars, driving drums
[Instrument: Clean Electric Guitar, Bass, Drums]
[Mood: Nostalgic] [Energy: Medium→High]
[Texture: Warm, Jangly]
```

**Soft Rock / Ballad**
```
Soft rock ballad, emotional and soaring, slow 75 BPM, powerful vocals, acoustic guitar, piano, strings
[Instrument: Acoustic Guitar, Piano, Strings, Light Drums]
[Mood: Emotional] [Energy: Low→High]
[Vocal Style: Powerful, Emotive]
```

### Electronic / Dance

**EDM / House**
```
House music, energetic and driving, 128 BPM, catchy vocal hooks, synths, four-on-the-floor
[Instrument: Synths, House Drums, Bass]
[Mood: Euphoric] [Energy: High]
[Vocal Style: Catchy, Bright]
```

**Lo-Fi / Chill**
```
Lo-fi chill, relaxed and hazy, 80 BPM, soft distant vocals, vinyl crackle, mellow keys
[Instrument: Mellow Keys, Lo-fi Drums, Vinyl Texture]
[Mood: Relaxed] [Energy: Low]
[Texture: Lo-fi, Hazy, Warm]
```

**Synthwave**
```
Synthwave, nostalgic and cinematic, 100 BPM, processed vocals, retro synths, pulsing bass
[Instrument: Retro Synths, Pulsing Bass, Electronic Drums]
[Mood: Nostalgic] [Energy: Medium→High]
[Texture: Cinematic, Retro]
```

### Country

**Modern Country**
```
Modern country, upbeat and feel-good, 120 BPM, warm twangy vocals, acoustic guitar, fiddle
[Instrument: Acoustic Guitar, Fiddle, Pedal Steel, Drums]
[Mood: Feel-good] [Energy: Medium→High]
[Vocal Style: Warm, Twangy]
```

**Country Ballad**
```
Country ballad, emotional and heartfelt, slow 65 BPM, sincere vocals, acoustic guitar, steel guitar
[Instrument: Acoustic Guitar, Pedal Steel, Light Drums]
[Mood: Heartfelt] [Energy: Low]
[Vocal Style: Sincere, Emotive]
```

### Jazz / Blues

**Jazz**
```
Jazz, smooth and sophisticated, 100 BPM swing, warm vocals, piano trio, brushed drums
[Instrument: Piano, Upright Bass, Brushed Drums]
[Mood: Sophisticated] [Energy: Medium]
[Vocal Style: Warm, Jazzy Phrasing]
```

**Blues**
```
Blues, raw and soulful, slow 65 BPM shuffle, gritty vocals, electric guitar, organ
[Instrument: Electric Guitar, Organ, Bass, Drums]
[Mood: Soulful] [Energy: Low→Medium]
[Vocal Style: Gritty, Soulful]
```

### Latin

**Reggaeton**
```
Reggaeton, infectious and sultry, 95 BPM, smooth Spanish vocals, dembow beat, synth bass
[Instrument: Dembow Drums, Synth Bass, Percussion]
[Mood: Sensual] [Energy: High]
[Vocal Style: Smooth, Confident]
```

**Latin Pop**
```
Latin pop, romantic and upbeat, 110 BPM, passionate vocals, acoustic guitar, percussion
[Instrument: Acoustic Guitar, Congas, Piano, Strings]
[Mood: Romantic] [Energy: Medium→High]
[Vocal Style: Passionate, Warm]
```

### Folk / Acoustic

**Folk**
```
Folk, intimate and storytelling, 95 BPM, warm natural vocals, acoustic guitar, light percussion
[Instrument: Acoustic Guitar, Mandolin, Light Percussion]
[Mood: Intimate] [Energy: Low→Medium]
[Vocal Style: Natural, Storytelling]
```

**Singer-Songwriter**
```
Singer-songwriter, vulnerable and honest, slow 70 BPM, intimate vocals, fingerpicked guitar, piano
[Instrument: Fingerpicked Guitar, Piano]
[Mood: Vulnerable] [Energy: Low]
[Vocal Style: Intimate, Honest]
```

### Gospel / Spiritual

**Gospel**
```
Gospel, uplifting and powerful, 100 BPM, soaring vocals with choir, organ, claps
[Instrument: Organ, Gospel Choir, Handclaps, Drums]
[Mood: Uplifting] [Energy: High]
[Vocal Style: Powerful, Soulful]
```

**Worship**
```
Contemporary worship, inspiring and reverent, 75 BPM, sincere vocals, atmospheric pads, acoustic guitar
[Instrument: Acoustic Guitar, Ambient Pads, Light Drums]
[Mood: Reverent] [Energy: Medium]
[Vocal Style: Sincere, Warm]
```

## Vocal styles

Direct the voice with 2-3 descriptors, effect tags, and phrasing tricks. Match the voice to the genre and vary it across the song.

### Voice character descriptors

**Texture**

| Descriptor | Effect | Best for |
|------------|--------|----------|
| `breathy` | Audible air, intimate | R&B, ballads, soft pop |
| `raspy` | Gritty texture | Rock, blues, emotional songs |
| `smooth` | Clean, polished | Pop, R&B, jazz |
| `warm` | Full, rich tone | Soul, folk, ballads |
| `bright` | Clear, forward | Pop, dance, upbeat |
| `dark` | Deep, moody | Electronic, gothic, intense |
| `airy` | Light, ethereal | Indie, dream pop |
| `velvety` | Soft, luxurious | Jazz, R&B, romantic |
| `nasal` | Sharp, cutting | Punk, indie, character |
| `husky` | Low, sensual | R&B, late-night jazz |

**Delivery style**

| Descriptor | Effect | Best for |
|------------|--------|----------|
| `powerful` | Strong projection | Anthems, rock, gospel |
| `intimate` | Close-mic, personal | Ballads, acoustic |
| `confident` | Assured, bold | Hip-hop, pop, rock |
| `vulnerable` | Fragile, emotional | Singer-songwriter |
| `aggressive` | Intense, forceful | Metal, trap, punk |
| `playful` | Light, fun | Pop, children's, upbeat |
| `sensual` | Seductive, slow | R&B, Latin, jazz |
| `haunting` | Eerie, atmospheric | Ambient, dark electronic |

**Vocal techniques**

| Descriptor | Effect | Best for |
|------------|--------|----------|
| `belting` | Loud, powerful high notes | Anthems, climaxes |
| `falsetto` | High, head voice | R&B, indie, dramatic |
| `whisper` | Very soft, intimate | Intros, bridges, ASMR |
| `spoken` | Talking, not singing | Hip-hop verses, intros |
| `harmonies` | Multi-voice layers | Gospel, pop, choruses |
| `ad-libs` | Spontaneous additions | R&B, hip-hop |

### Voice type combinations

**Female vocals**

```
Pop Princess    [Female Vocal] [Vocal Style: Bright, Clear, Polished]
R&B Diva        [Female Vocal] [Vocal Style: Smooth, Breathy, Runs and Riffs]
Indie Darling   [Female Vocal] [Vocal Style: Airy, Intimate, Slightly Nasal]
Rock Power      [Female Vocal] [Vocal Style: Raw, Powerful, Raspy Edge]
Folk Singer     [Female Vocal] [Vocal Style: Warm, Natural, Storytelling]
Jazz Chanteuse  [Female Vocal] [Vocal Style: Velvety, Warm, Jazzy Phrasing]
```

**Male vocals**

```
Pop Star        [Male Vocal] [Vocal Style: Bright, Smooth, Falsetto Touches]
R&B Crooner     [Male Vocal] [Vocal Style: Silky, Intimate, Runs and Riffs]
Rock Singer     [Male Vocal] [Vocal Style: Raspy, Powerful, Raw]
Rapper          [Male Vocal] [Vocal Style: Confident, Rhythmic, Clear Diction]
Country Voice   [Male Vocal] [Vocal Style: Warm, Twangy, Sincere]
Soul Singer     [Male Vocal] [Vocal Style: Rich, Powerful, Gospel Influence]
```

### Vocal effect tags

**Reverb & space**

```
[Vocal Effect: Reverb]        Standard room ambience
[Vocal Effect: Large Reverb]  Cathedral/stadium sound
[Vocal Effect: Dry]           No reverb, upfront
[Vocal Effect: Close-mic]     Intimate, in-your-ear
```

**Special effects**

```
[Vocal Effect: Echo]          Repeated delays
[Vocal Effect: Auto-tune]     Pitch correction effect
[Vocal Effect: Distortion]    Gritty, overdriven
[Vocal Effect: Vocoder]       Robotic/synthetic
[Vocal Effect: Telephone]     Lo-fi, filtered
```

### Pronunciation & phrasing tricks

**Extending notes** — stretch the vowel to hold a note:
- `loooove` — stretches the vowel
- `niiight` — holds the note longer
- `waaaaay` — sustained, dramatic

**Emphasizing consonants**:
- `runnn` — sustained ending
- `kissss` — hissing emphasis
- `stoppp` — hard ending

**Creating pauses**:
- Commas `,` for brief pauses
- Ellipses `...` for longer breath/pause
- Hyphens `-` for connected syllables
- Line breaks = natural breathing points

**Phrasing examples**

Smooth / connected:
```
Holding you close tonight
```

Broken / emotional:
```
Holding... you... close... tonight
```

Punchy / rhythmic:
```
Hold-ing, you, close, to-night
```

### Full vocal direction examples

Intimate R&B ballad:
```
[Intro]
[Female Vocal] [Vocal Style: Breathy, Intimate, Close-mic]
[Vocal Effect: Warm Reverb]
```

Power pop anthem:
```
[Chorus] [Energy: High]
[Female Vocal] [Vocal Style: Powerful, Bright, Belting]
[Vocal Effect: Large Reverb]
```

Hip-hop verse:
```
[Verse]
[Male Vocal] [Vocal Style: Confident, Clear Diction, Rhythmic]
[Vocal Effect: Dry]
```

Emotional rock bridge:
```
[Bridge] [Mood: Melancholic]
[Male Vocal] [Vocal Style: Raspy, Vulnerable, Building to Powerful]
```

Dreamy indie:
```
[Verse]
[Female Vocal] [Vocal Style: Airy, Ethereal, Whispered]
[Vocal Effect: Heavy Reverb, Delay]
```

Gospel climax:
```
[Chorus] [Energy: High]
[Vocal Style: Powerful, Soulful, Belting]
[Duet with Gospel Choir]
[Vocal Effect: Large Reverb, Harmonies]
```

### Vocal tips

1. **Pick 2-3 descriptors max** — too many conflict.
2. **Match voice to genre** — raspy for rock, smooth for R&B.
3. **Vary throughout the song** — soft verse, powerful chorus.
4. **Use contrasts** — whisper before a belt for impact.
5. **Consider the lyrics** — emotional words need emotional delivery.

## Viral hooks

Apply these when the user wants catchy, viral, earworm, or TikTok-optimized songs.

### Earworm science

- 4 syllables max per hook phrase (brain chunking limit).
- 1.5-2 second phonological loop capacity.
- Repetition matters MORE than melody — repeat the hook 15-20+ times.
- 73.7% of earworms have lyrics (vs 7.7% instrumental) — put words on the hook.
- The hook must appear in the FIRST 7 SECONDS (streaming skip threshold).

### Optimal parameters

- 120-140 BPM is the sweet spot for earworms.
- Open vowels (oh, ah, ee, oo) are easier to sing and stick harder.
- Short hook: 3-5 words max for the core phrase.

### Psychological techniques

- **Zeigarnik Effect**: Incomplete phrases with "..." — the brain replays them to resolve the tension.
- **Dopamine architecture**: Quiet bridge → build → MAXIMUM chorus. Contrast = dopamine.
- **Call and response**: "You feel stuck?" (Stuck!) — creates participation and TikTok duet potential.
- **Na na na sections**: Universal singalong, no language barrier.
- **Pattern + surprise**: Predictable chorus, unexpected build/break.

### Suno-specific tags for catchiness

- Add "catchy hook" and "earworm" to the style prompt.
- Use a `[Catchy Hook]` tag before the chorus.
- CAPS on hook lines signals Suno to emphasize them.
- Add "gang vocals" for chorus power.
- Add "singalong" and "anthem" to the tags.
- Add "millennial whoop" for the oh-oh-oh pattern.

### Lyric techniques

- Rhyme schemes aid memory.
- Conversational language feels relatable.
- Unfinished phrases in the bridge create tension.
- Repeat the title/hook constantly.

## Generate it

Once the lyrics are written, produce audio with the `suno` CLI. Map the guide's output to flags:

- **Style Prompt line** → `--tags`
- **Lyric block** (the `[Verse]` / `[Chorus]` / meta-tagged text) → `--lyrics-file` (or `--lyrics "..."` inline)
- **Title** → `--title`

Save the lyric block to a plain `.txt` file (it stays plain text — no markdown), then:

```
suno generate \
  --title "Weekend Code" \
  --tags "indie rock, jangly and nostalgic, 110 BPM, warm male vocal, clean guitars, driving drums" \
  --lyrics-file lyrics.txt \
  --model v4.5-all \
  --wait --download ./
```

Use `--model v4.5-all` for cheap drafts (~10 credits/call — the cheapest model, ideal for iterating on structure and hooks). When the song is right, re-run on a stronger model. `--wait` polls until the render is done; `--download ./` pulls the MP3 into the current directory and embeds the lyrics into the file's metadata.

**Full `suno generate` flags** (v0.8.0):

```
--title <str>              Song title
--tags <str>               Style prompt / genre tags
--exclude <str>            Styles to steer away from
--lyrics <str>             Inline lyrics
--lyrics-file <path>       Lyrics from a file (conflicts with --lyrics)
--model <ver>             v5.5 | v5 | v4.5+ | v4.5 | v4.5-all | v4
--vocal <male|female>      Vocal gender
--weirdness <0-100>        Experimentation / deviation
--style-influence <0-100>  How hard the tags steer the result
--audio-influence <0-100>  Weight of a reference audio input
--instrumental             No vocals
--persona <uuid>           Reuse a saved voice persona
--wait                     Block until the render finishes
--download <dir>           Download the finished MP3 (embeds lyrics)
--token <str>              Override the auth token
--no-captcha               Skip the captcha solve step
--force                    Bypass the duplicate-run guard and the placeholder preflight
```

**Cost**: v5.5 ≈ 70 credits/call; v4.5-all ≈ 10 credits/call (cheapest — use it for drafts). Downloading embeds the lyrics into the MP3.

### Quick alternatives

- `suno lyrics --prompt "..."` — a FREE lyric draft (no credits, no audio). Use it to spin up or refine a lyric block before generating.
- `suno describe --prompt "..."` — Suno writes the lyrics for you from a short description, then generates. Fastest path when you don't want to hand-write the song.
