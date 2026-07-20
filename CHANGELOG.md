# Changelog

## v0.8.0 — the composer and the renderer agree about the artifact

One invariant now holds end to end: the file named by the emitted generate command exists, is directly consumable by `--lyrics-file`, contains no unresolved instructions, and reflects every selected control.

**Breaking** (agents reading `write --json`):

- `write --out FILE` writes the **lyric block only**. It previously wrote a composite document (title, Style Prompt, `---` rules, Suno Tags, and in priming mode the Prime-Stack Map and research artefact) that the emitted workflow then handed to `--lyrics-file` — so headers, tags and research metadata were sent to Suno as lyrics and embedded in the MP3. The composite document moved to the new `--project-out FILE`.
- The `generate` string field is replaced by `next_action: {argv, command}`. `argv` is authoritative; `command` is shell-escaped display text. It is `null` when no `--out` file exists, instead of advertising a hardcoded `song.txt` that was never written.
- `write --mode priming` requires `--target`, `--objective` and `--domain` (exit 3 when missing) — priming is consent-based and every run must be auditable. `--domain` and `--subtlety` values are validated.
- `guide` lost the `write` alias (it competed with the `suno write` command).

**Fixes:**

- `--mood` / `--vocal` / `--bpm` / `--instrumental` now drive the Style Prompt, the `[Mood:]`/`[Energy:]`/vocal meta-tags and `suno_tags` from one resolved-controls struct. `--mood "dark and brooding"` no longer emitted `[Mood: Uplifting]` and an "uplifting" tag alongside it.
- `--instrumental` is coherent: no `<...>` fill instructions, `--instrumental` in the emitted command, no vocal-only tags.
- Titles and paths in the emitted command are shell-escaped (`She Said "Go"` produced invalid shell).
- `generate` refuses lyrics containing unresolved `<...>` scaffold placeholders (exit 3, naming the line numbers) so an unfilled draft cannot burn ~70 credits. `--force` overrides.
- The emitted command no longer pins `--model v4.5-all` while help and config advertise v5.5 — it omits `--model` so the configured default applies, and names the cheap-draft option separately.
- New fields: `placeholders_remaining`, `ready_to_generate`, `missing_requirements`, `project_written`.

**Discovery:**

- `write` leads the command list in `--help`; the root example is the full write → fill → generate → download flow, with a one-liner distinguishing write/generate/describe/lyrics. README Quick Start mirrors it.
- `write --help` no longer claims plain text on stdout while the framework sends JSON when piped: shell redirection gets the envelope, `--out` gets the lyrics file.
- `agent-info` gained the `write` output schema, workflow, and mode-specific required fields.

## v0.6.0 — framework conformance, captcha preflight, real config

**Breaking** (agents pinned to the 0.5.x contract must update):

- Exit codes remapped to the [agent-cli-framework](https://github.com/paperfoot/agent-cli-framework) contract: 0 success, 1 transient, 2 config/auth (auth was 3), 3 bad input incl. not-found (was 5), 4 rate limited. Code 5 removed.
- `list --json` data is now `{clips, next_cursor, has_more}` (was a bare clip array); `list --page` replaced by `--cursor <token>` (the old page numbers never worked against feed/v3's opaque cursors).
- `generate --variation` removed — it was parsed and silently ignored.
- `download --json` data is now `{downloaded, failed}` with `partial_success` status when some clips fail (was a bare path array).
- `delete`/`auth`/`set`/`publish`/`config` now emit success envelopes in JSON mode.

**Captcha & generation:**

- Captcha preflight: every gated command asks `/api/c/check` first and skips the Chrome solver entirely when the account isn't captcha-gated (most aren't).
- `extend`/`cover`/`remaster` now route through the same captcha pipeline as `generate`/`describe` (they previously posted `token: null` and failed on captcha-enforced accounts), and gained `--token`/`--no-captcha`.
- Fixed the bare `__client` cookie being dropped on the solver's cookie replay — the root cause of "hcaptcha never finished loading" on sub-threshold accounts.
- `--wait` now exits non-zero when generation fails (moderation rejections exit 3); previously failed clips exited 0.
- New model `v4.5-all` (chirp-auk-turbo, Suno's "best free model"); `extend` gained `--model`; new `--audio-influence` slider on generate/cover.
- Documented real credit costs: ≈70 credits per v5.5 call (35/clip), not ~10.

**Tooling:**

- `suno doctor` — auth/JWT/Chrome/API/credits/captcha health checks.
- `suno skill install|status` — agent skill for Claude Code, Codex CLI, Gemini CLI (replaces `install-skill`, which remains a hidden alias).
- `suno update` is distribution-aware: brew- and cargo-owned binaries are never self-replaced; the owner channel's upgrade command is returned instead.
- Config layer is now real: TOML file + `SUNO_*` env vars actually drive polling, default model, and download dir (`config show|set|path|check`).
- Duplicate-run guard on generate/describe/cover/remaster/update (`--force` bypasses).
- Vendored framework conformance probe + schemas under `conformance/`, run in CI; integration test suite under `tests/`.
- Fixed a UTF-8 panic when tables truncated multi-byte (CJK/emoji) prompts.

## v0.5.x

- v0.5.7 — fix captcha desktop viewport
- v0.5.6 — fix captcha cookie replay
- v0.5.5 — auth hardening and release cleanup
- v0.5.4 — auto-solve hCaptcha via piloted Chrome (CDP)
- v0.5.3 — add `suno auth --refresh` + clearer captcha-rollout error
- v0.5.2 — add in-process JWT refresh retry
- v0.5.1 — rename package to `suno`, add `install-skill` command
- v0.5.0 — fix cover/remaster endpoints, add persona + info commands, framework alignment

## Earlier

- v0.4.0 — zero-friction auth: `suno auth --login`
- v0.3.0 — audit fixes, set metadata, publish, timed lyrics, ID3 embedding
- v0.2.0 — search, delete, slug filenames, renamed commands
- v0.1.0 — initial release
