---
name: suno
description: The complete Suno tool — write and generate AI music from the terminal with the `suno` CLI. Use when the user asks to generate/make/create a song, music, a track, or audio; to write a song or lyrics for Suno; for catchy/viral/earworm hooks; for covers, remasters, stems, or voice personas; to download Suno songs (auto-embeds lyrics into MP3); or for priming/subliminal/charm/research songs — "prime [name] to [action]", subliminal song, charm round, research priming, or a song "for the paper". `suno write` scaffolds the song from a built-in grammar, `suno generate` renders the audio. Run `suno agent-info` for the full machine-readable capability dump.
---

# suno CLI

One binary does the whole job — composing the song and rendering the audio. All capability detail lives in the binary so it never drifts from this file.

## Make a song

1. Scaffold a Suno-ready song from the built-in grammar:
   ```bash
   suno write --genre "indie rock" --theme "late-night city drives" --vocal male --viral --out song.txt
   ```
   You get a Style Prompt line, a meta-tagged `[Verse]`/`[Chorus]` skeleton with inline `<...>` lyric placeholders, a Suno Tags line, and the exact `suno generate` command to run next.
2. Fill in the `<...>` lyric lines in `song.txt`.
3. Render the audio:
   ```bash
   suno generate --title "..." --tags "..." --lyrics-file song.txt --model v4.5-all --wait --download ./songs/
   ```

## Priming / research songs

```bash
suno write --mode priming --target "..." --objective "..." --domain investment --subtlety stealth --out song.txt
```
Adds a chill-lounge low-arousal scaffold plus a Prime-Stack Map and research-artefact block. Then fill the lyrics and `suno generate` as above.

## Deep reference & everything else

- `suno guide songwriting` — the full grammar (structure, meta-tags, genres, vocal styles, viral hooks)
- `suno guide priming` — consent frame, evidence-graded prime library, phonetic name-embedding, quality gates
- `suno agent-info` — machine-readable manifest: every command, flag, model, exit code, envelope shape, config key
- `suno --help` / `suno <command> --help` — usage, tips, real examples
- First run: `suno auth --login`, then `suno doctor` to verify

Piped output is a JSON envelope automatically. `suno write` and `suno lyrics` are free; generation ≈70 credits/call on v5.5.
