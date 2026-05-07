# Everest Capital — ClaudeTerminal Fork Baseline

This repository is a fork of [`talayash/claude-terminal`](https://github.com/talayash/claude-terminal) maintained by **Everest Capital USA** for the BidDeed.AI / ZoneWise.AI ecosystem.

## Lineage
- **Upstream:** talayash/claude-terminal (MIT, Tier 0 / PERMISSIVE_FREE)
- **Forked from commit:** `cd0d62cc3141ccc5d3cf21ad7a393dfa66c30079`
- **Pinned upstream version:** v1.20.7 (2026-05-04)
- **Forked on:** 2026-05-06
- **Forked by:** Ariel Shapira (Everest Capital USA)

## REPOEVAL summary
- **Verdict:** ADOPT
- **Score:** 86 / 100 (threshold ≥75)
- **License:** MIT (confidence 1.00 — raw LICENSE file verified verbatim)
- **Reference:** `extrep_evaluations.id = 97a77546-e1dd-4a2a-8d36-80564859e180` in Everest Supabase

## Changes since fork
1. **Disabled upstream auto-update** (commit `95e8d12a`).
2. **Added BASELINE.md** (commit `95e8d12a`).
3. **Wired profile system to `everest_tenants` SSOT** (commit `5034f136`).
4. **Wired hints panel to skill catalog** (commit `9ebb7240`).
5. **Added CI release pipeline** (commit `ed515e9b`).
6. **Removed upstream's incompatible `release.yml`** (commit `75420bd3`).
7. **Telegram exit-code hook** (this commit): `src-tauri/src/telegram_hook.rs` watches every spawned child process and notifies on abnormal exit.

## Adoption plan (status as of 2026-05-07)
- [x] Fork to `breverdbidder/claude-terminal`
- [x] Pin baseline at upstream `cd0d62cc...`
- [x] Disable upstream auto-update endpoint
- [x] Wire profile system → `everest_tenants` SSOT (8 default profiles)
- [x] Wire hints panel → skill catalog (7 categories, 29 hints)
- [x] CI release workflow + first tag `v1.20.7-everest.1` pushed
- [x] Telegram exit-code hook (this commit; will appear in `v1.20.7-everest.2` build)
- [ ] Re-tag as `everest-cct-1.20.7` once all integration changes are battle-tested

## Telegram exit-code notifications
The fork ships a per-child exit watcher at `src-tauri/src/telegram_hook.rs`. For every PTY-spawned process (Claude Code session, npm script, raw shell), it spawns a tokio background task that waits on `child.wait()` and sends a Telegram message when the process exits with a code other than:
- `0`   — clean exit
- `130` — SIGINT (Ctrl+C)
- `143` — SIGTERM (user closed the terminal cell)

These three codes are filtered to avoid notification spam from normal use.

### Setup (one time, on each machine running ClaudeTerminal)
The hook reads two env vars **at notification time** (not at startup, so changes take effect on the next abnormal exit):

```powershell
# Required: bot API token
[Environment]::SetEnvironmentVariable("TELEGRAM_BOT_TOKEN", "<your_bot_token>", "User")

# Optional: chat ID (defaults to 740118343 = BidDeedAI_bot DM with Ariel)
[Environment]::SetEnvironmentVariable("TELEGRAM_CHAT_ID", "740118343", "User")
```

Restart ClaudeTerminal after setting these. If `TELEGRAM_BOT_TOKEN` is unset or empty, the hook silently no-ops — no errors, no spam.

### Notification payload
Each notification includes:
- Description (terminal nickname → label → "shell (<label>)" → "npm run <script>")
- Short terminal ID (first 8 chars of UUID)
- Exit code
- Host (COMPUTERNAME on Windows, HOSTNAME on Linux/macOS)

### Implementation notes
- **Non-blocking**: terminal creation never waits for the watcher. The watcher is spawned via `Handle::try_current().spawn(...)` and is fire-and-forget.
- **No new deps**: uses existing `reqwest` (already in Cargo.toml) and `tokio` (already required by Tauri 2).
- **No runtime panics**: if `tokio::runtime::Handle::try_current()` returns Err (no tokio context), the watcher logs to stderr and returns — does NOT panic.
- **5-second timeout** on the Telegram POST; failures log to stderr and are otherwise ignored.

## CI release pipeline
`.github/workflows/release-windows.yml` builds and releases the Windows installer on every `v*` tag push. See commit `ed515e9b` for details. Releases land at https://github.com/breverdbidder/claude-terminal/releases.

To release a build with this commit's Telegram hook included:
```powershell
git tag v1.20.7-everest.2
git push origin v1.20.7-everest.2
```

## Refreshing the tenant seed
Snapshot of `everest_tenants` in `seeds/everest-tenants.json`. To refresh: re-run the export SQL (in commit `5034f136`), replace the file, push a new tag.

## Refreshing the hint seed
Hand-curated `seeds/everest-hints.json` with 29 hints across 7 categories matching `everest_tenants.skill_allowlist`. There is no dedicated `everest_skills` table.

## What gets seeded (profiles)
8 tenant profiles: everest-capital, biddeed, zonewise, brevard-doors, property360, kenstrekt, protection-partners, everest-portfolio. Each carries `working_directory`, `claude_args`, and `env_vars` (`EVEREST_TENANT`, `EVEREST_BUSINESS_KIND`, `EVEREST_DOMAIN`, `EVEREST_STAGE`, `EVEREST_PAIRING_RULE`).

## What gets exposed (hints)
29 hints in 7 Everest categories (SEO 5, Conversion 7, Copy 4, Outbound 3, Strategy 5, RevOps 4, Growth 1), appended after the upstream defaults in F1 panel.

## Update strategy (upstream)
Manual: cherry-pick or merge from upstream master into a feature branch, re-run REPOEVAL, update BASELINE.md, push a new tag.

## Honesty tags (this revision)
- VERIFIED: All 3 spawn sites in `terminal.rs` rewired (`_child` → `child`); `watch_child_exit` called immediately after `let id = ...`. 0 leftover `_child` bindings.
- VERIFIED: `mod telegram_hook;` declared in `main.rs` after `mod everest_hints;`.
- VERIFIED: No Cargo.toml change required — `reqwest` (with rustls-tls + json) and `tokio` (with full features) already present.
- INFERRED: `tokio::runtime::Handle::try_current()` returns Ok inside Tauri command handlers because Tauri runs commands on its tokio runtime; `Err` path is defensive for any caller outside that context.
- INFERRED: Filtering exit codes 0/130/143 covers the common user-driven exits without missing genuine crashes (signal-killed processes report 128+signal; SIGSEGV=139, SIGABRT=134, OOM-kill=137 will all notify).
- UNTESTED: Build of this commit (will appear when `v1.20.7-everest.2` tag is pushed).
- UNTESTED: An actual non-zero exit triggering an actual Telegram message (requires running build with the env vars set).

## Contact
**Owner:** Everest Capital USA / Ariel Shapira
**Origin context:** postmortem session 2026-05-06/07 (summit_chat_dispatch dispatcher recovery + parallel REPOEVAL + adoption plan steps 1-5 + CI release pipeline + Telegram exit hook)
