# Changelog

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
