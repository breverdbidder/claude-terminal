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
- **Method:** parallel GitHub-API REPOEVAL via `pg_net` + vault-stored PAT
- **Reference:** `extrep_evaluations.id = 97a77546-e1dd-4a2a-8d36-80564859e180` in Everest Supabase

## Changes since fork
1. **Disabled upstream auto-update** in `src-tauri/tauri.conf.json` (commit `95e8d12a`):
   - `bundle.createUpdaterArtifacts: true` → `false`
   - `plugins.updater.endpoints: [...]` → `[]`
2. **Added BASELINE.md** (commit `95e8d12a`).
3. **Wired profile system to `everest_tenants` SSOT** (this commit):
   - New file `seeds/everest-tenants.json`: 8 tenant profiles snapshotted from Supabase.
   - New module `src-tauri/src/everest_seed.rs`: compile-time embeds the JSON via `include_str!` and seeds the local SQLite `profiles` table on first launch only.
   - Hook in `src-tauri/src/main.rs` `setup()` calls `everest_seed::seed_default_profiles_if_empty(&db)` right after DB init.

## Adoption plan (status as of 2026-05-06)
- [x] Fork to `breverdbidder/claude-terminal` (HTTP 202)
- [x] Pin baseline at upstream `cd0d62cc...`
- [x] Disable upstream auto-update endpoint
- [x] Wire profile system → `everest_tenants` SSOT (8 default profiles seeded)
- [ ] Wire hints panel → query `everest_skills` table at startup (replace `config::get_default_hints()`)
- [ ] Add Telegram exit-code hook (`src-tauri/src/terminal.rs` on_exit handler)
- [ ] Re-tag fork as `everest-cct-1.20.7` once integration changes are merged

## Refreshing the tenant seed
The seed in `seeds/everest-tenants.json` is a snapshot of the Supabase SSOT taken at the timestamp in `_meta.generated_at`. To refresh:

1. Connect to the Everest Supabase project (`mocerqjnksmhcjzxrewo`).
2. Re-run the export (the SQL is in this commit's history).
3. Replace `seeds/everest-tenants.json` with the new payload.
4. `cargo build --release` to re-embed.
5. Existing user installs are unaffected — the seeder only runs when the local profiles table is empty. New installs pick up the refreshed defaults automatically.

## What gets seeded
The eight tenant profiles match `public.everest_tenants` rows by slug:
`everest-capital`, `biddeed`, `zonewise`, `brevard-doors`, `property360`, `kenstrekt`, `protection-partners`, `everest-portfolio`.

Each profile carries:
- `working_directory` defaulting to `%USERPROFILE%\Code\<slug>` (user-editable)
- `claude_args: ["--model", "opus"]`
- `env_vars`: `EVEREST_TENANT`, `EVEREST_BUSINESS_KIND`, `EVEREST_DOMAIN`, `EVEREST_STAGE`, `EVEREST_PAIRING_RULE` — surfaced to every Claude Code session launched from that profile

## Update strategy (upstream)
This fork tracks upstream **manually**. To pull a newer upstream release:
1. Inspect new commits on `talayash/claude-terminal` master.
2. Cherry-pick or merge into a feature branch off this fork's master.
3. Re-run REPOEVAL on the new HEAD before merging.
4. Update this `BASELINE.md` with the new pinned commit and date.

## Honesty tags (this revision)
- VERIFIED: 8 tenant rows in `public.everest_tenants` reflected verbatim in `seeds/everest-tenants.json`.
- VERIFIED: `ConfigProfile` struct shape in `src-tauri/src/config.rs` matches the seeder's deserialization target.
- VERIFIED: `mod everest_seed;` and seed call were inserted into `src-tauri/src/main.rs` at the correct site.
- UNTESTED: `cargo build --release` (Rust code not compiled in this session).
- UNTESTED: First-launch behavior in the actual desktop app (binary not run).

## Contact
**Owner:** Everest Capital USA / Ariel Shapira
**Origin context:** postmortem session 2026-05-06 (summit_chat_dispatch dispatcher recovery + parallel REPOEVAL + adoption plan steps 1-4)
