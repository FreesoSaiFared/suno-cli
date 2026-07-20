---
name: suno
description: Generate AI music from the terminal using the `suno` CLI. Use when user asks to "generate a song", "make music", "create AI music", "make a track", "generate audio", "write a song/lyrics for Suno", or wants to programmatically use Suno for custom lyrics, tags, voice personas, covers, remasters, or stems. Also use when downloading Suno songs (auto-embeds lyrics into MP3). For how to write for Suno, run `suno guide songwriting`; run `suno agent-info` for the full machine-readable capability dump.
---

# suno CLI

All capability detail lives in the binary, so it never drifts from this file:

- `suno agent-info` — machine-readable manifest: every command, flag, model, exit code, envelope shape, config key
- `suno --help` / `suno <command> --help` — usage, tips, and real examples
- First run: `suno auth --login`, then `suno doctor` to verify the setup
- Piped output is a JSON envelope automatically; `lyrics` is free, generation ≈70 credits/call on v5.5
- Writing a song? `suno guide` lists built-in guides — `suno guide songwriting` (the grammar) and `suno guide priming`
