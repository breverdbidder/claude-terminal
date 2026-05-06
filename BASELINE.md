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
1. **Disabled upstream auto-update** in `src-tauri/tauri.conf.json` (commit `95e8d12a`).
2. **Added BASELINE.md** (commit `95e8d12a`).
3. **Wired profile system to `everest_tenants` SSOT** (commit `5034f136`):
   - `seeds/everest-tenants.json` snapshots 8 tenant profiles from Supabase.
   - `src-tauri/src/everest_seed.rs` seeds the SQLite profiles table on first launch (no-op if any profile already exists).
   - `src-tauri/src/main.rs` calls the seeder in `setup()` after `Database::new()`.
4. **Wired hints panel to skill catalog** (this commit):
   - `seeds/everest-hints.json`: 7 categories, 29 hints — covers every distinct slug in the union of `everest_tenants.skill_allowlist`.
   - `src-tauri/src/everest_hints.rs`: compile-time embeds the JSON via `include_str!` and exposes `load_everest_categories()`.
   - `src-tauri/src/config.rs`: `get_default_hints()` now returns upstream defaults **plus** the Everest categories. The original upstream body was moved to a private `upstream_default_hints()` helper. Failure to load the Everest seed is non-fatal.
   - `src-tauri/src/main.rs`: declares `mod everest_hints;`.
   - Each Everest hint command references `$EVEREST_TENANT` and `$EVEREST_DOMAIN` (env vars set by the tenant profile from step 3).

## Adoption plan (status as of 2026-05-06)
- [x] Fork to `breverdbidder/claude-terminal` (HTTP 202)
- [x] Pin baseline at upstream `cd0d62cc...`
- [x] Disable upstream auto-update endpoint
- [x] Wire profile system → `everest_tenants` SSOT (8 default profiles seeded)
- [x] Wire hints panel → skill catalog (7 categories, 29 hints; backed by `everest_tenants.skill_allowlist`)
- [ ] Add Telegram exit-code hook (`src-tauri/src/terminal.rs` on_exit handler — BidDeedAI_bot, chat_id 740118343)
- [ ] Re-tag fork as `everest-cct-1.20.7` once integration changes are merged

## Refreshing the tenant seed
The seed in `seeds/everest-tenants.json` is a snapshot of the Supabase SSOT taken at `_meta.generated_at`. To refresh: connect to project `mocerqjnksmhcjzxrewo`, re-run the export (SQL is in commit `5034f136`), replace `seeds/everest-tenants.json`, and `cargo build --release`. Existing user installs are unaffected; new installs pick up refreshed defaults.

## Refreshing the hint seed
The seed in `seeds/everest-hints.json` is hand-curated. There is **no dedicated `everest_skills` table** in Supabase; the canonical slug list is the union of `public.everest_tenants.skill_allowlist`. To refresh:

1. Run `SELECT DISTINCT unnest(skill_allowlist) FROM public.everest_tenants ORDER BY 1;`
2. Compare against the `categories[].hints[].title` set in `seeds/everest-hints.json`. Add or retire hints to match.
3. Update `_meta.generated_at`.
4. `cargo build --release` to re-embed.
5. The hints panel always merges these with the upstream defaults; if the JSON parse fails, upstream defaults still render and a warning prints to stderr.

## What gets seeded (profiles)
The eight tenant profiles match `public.everest_tenants` rows by slug:
`everest-capital`, `biddeed`, `zonewise`, `brevard-doors`, `property360`, `kenstrekt`, `protection-partners`, `everest-portfolio`.

Each profile carries:
- `working_directory` defaulting to `%USERPROFILE%\Code\<slug>` (user-editable)
- `claude_args: ["--model", "opus"]`
- `env_vars`: `EVEREST_TENANT`, `EVEREST_BUSINESS_KIND`, `EVEREST_DOMAIN`, `EVEREST_STAGE`, `EVEREST_PAIRING_RULE`

## What gets exposed (hints)
The 29 hints are grouped into 7 Everest categories, appended after the upstream defaults:
- **Everest · SEO** (5): programmatic-seo, seo-audit, ai-seo, schema-markup, site-architecture
- **Everest · Conversion** (7): page-cro, form-cro, popup-cro, paywall-upgrade-cro, signup-flow-cro, lead-magnets, free-tool-strategy
- **Everest · Copy & Content** (4): copywriting, copy-editing, content-strategy, social-content
- **Everest · Outbound** (3): cold-email, email-sequence, paid-ads
- **Everest · Strategy & Research** (5): launch-strategy, customer-research, competitor-alternatives, pricing-strategy, marketing-psychology
- **Everest · RevOps & Analytics** (4): sales-enablement, revops, analytics-tracking, ab-test-setup
- **Everest · Growth Programs** (1): referral-program

## Update strategy (upstream)
This fork tracks upstream **manually**. Inspect new commits on `talayash/claude-terminal` master, cherry-pick or merge into a feature branch off this fork's master, re-run REPOEVAL on the new HEAD before merging, and update this `BASELINE.md`.

## Honesty tags (this revision)
- VERIFIED: 29 distinct slugs from `everest_tenants.skill_allowlist` correspond 1:1 to 29 hints across 7 categories.
- VERIFIED: `Hint` and `HintCategory` struct shapes in `src-tauri/src/config.rs` match the seeder's deserialization target.
- VERIFIED: `mod everest_hints;` inserted in `main.rs`; `get_default_hints` wrapped in `config.rs` with original body moved to `upstream_default_hints` helper.
- INFERRED: There is no `everest_skills` table; the canonical SSOT for slugs is `everest_tenants.skill_allowlist`. Hint commands and descriptions are hand-written for this commit.
- UNTESTED: `cargo build --release` (Rust code not compiled in this session).
- UNTESTED: F1 panel actually rendering the new categories with correct icons.

## Contact
**Owner:** Everest Capital USA / Ariel Shapira
**Origin context:** postmortem session 2026-05-06 (summit_chat_dispatch dispatcher recovery + parallel REPOEVAL + adoption plan steps 1-5)
