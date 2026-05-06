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

## Changes in this baseline commit
1. **Disabled upstream auto-update** in `src-tauri/tauri.conf.json`:
   - `bundle.createUpdaterArtifacts: true` → `false`
   - `plugins.updater.endpoints: [...]` → `[]`
   - Reason: upstream's endpoint points at `talayash/claude-terminal/releases/latest/download/latest.json` plus the upstream Cloudflare Worker `ct-analytics.claude-terminal.workers.dev/update`. Leaving these in would silently roll customizations forward and fight Everest patches.
2. **Added this `BASELINE.md`** so fork intent + lineage are discoverable from the repo root.

## Adoption plan (status as of 2026-05-06)
- [x] Fork to `breverdbidder/claude-terminal` (HTTP 202)
- [x] Pin baseline at upstream `cd0d62cc...`
- [x] Disable upstream auto-update endpoint
- [ ] Wire profile system → `everest_tenants` SSOT (8 default profiles, one per tenant)
- [ ] Wire hints panel → query `everest_skills` table at startup
- [ ] Add Telegram exit-code hook (`src-tauri/src/terminal.rs` on_exit handler — BidDeedAI_bot, chat_id 740118343)
- [ ] Re-tag fork as `everest-cct-1.20.7` once integration changes are merged

## Update strategy
This fork tracks upstream **manually**. To pull a newer upstream release:
1. Inspect new commits on `talayash/claude-terminal` master
2. Cherry-pick or merge into a feature branch off this fork's master
3. Re-run REPOEVAL on the new HEAD before merging
4. Update this `BASELINE.md` with the new pinned commit and date

## Tenant model
The 8 Everest tenants (`everest-capital`, `biddeed`, `zonewise`, `brevard-doors`, `property360`, `kenstrekt`, `protection-partners`, `everest-portfolio`) map 1:1 to ClaudeTerminal profiles. Each tenant profile carries its repo, branch, env vars, and CLI flags.

## Contact
**Owner:** Everest Capital USA / Ariel Shapira
**Origin context:** postmortem session 2026-05-06 (summit_chat_dispatch dispatcher recovery + parallel REPOEVAL)
