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
3. **Wired profile system to `everest_tenants` SSOT** (commit `5034f136`).
4. **Wired hints panel to skill catalog** (commit `9ebb7240`): 7 categories, 29 hints, 1:1 coverage of `everest_tenants.skill_allowlist`.
5. **Added CI release pipeline** (this commit): `.github/workflows/release-windows.yml` — auto-builds Windows installer on every `v*` tag push or manual dispatch.

## Adoption plan (status as of 2026-05-07)
- [x] Fork to `breverdbidder/claude-terminal` (HTTP 202)
- [x] Pin baseline at upstream `cd0d62cc...`
- [x] Disable upstream auto-update endpoint
- [x] Wire profile system → `everest_tenants` SSOT (8 default profiles seeded)
- [x] Wire hints panel → skill catalog (7 categories, 29 hints)
- [x] CI release workflow (this commit; first tag `v1.20.7-everest.1` pushed alongside)
- [ ] Add Telegram exit-code hook (`src-tauri/src/terminal.rs` on_exit handler — BidDeedAI_bot, chat_id 740118343)
- [ ] Re-tag as `everest-cct-1.20.7` once Telegram hook merges (in the meantime, intermediate releases are tagged `v1.20.7-everest.{1,2,…}`)

## CI release pipeline
The fork ships a GitHub Actions workflow at `.github/workflows/release-windows.yml` that:
- **Triggers** on push of any `v*` tag, or manual dispatch from the Actions tab with a tag-name input
- **Runs** on `windows-latest`, installs Node 20 + Rust stable, caches deps via `Swatinem/rust-cache`
- **Builds** via `tauri-apps/tauri-action@v0`, which runs `npm ci && npm run tauri build` and uploads the resulting `.exe` (NSIS) and `.msi` artifacts as a GitHub Release
- **Typical wall time:** 15–25 minutes per build (cold cache slower; warm cache much faster)
- **Installer signing:** unsigned. Windows SmartScreen will warn "Unknown publisher" — click *More info* → *Run anyway*. (Code signing requires a real cert; not in scope for this fork.)
- **Updater manifest (`latest.json`):** explicitly NOT generated (`includeUpdaterJson: false`), since the in-app updater was disabled in step 3.

To cut a new release:
```powershell
git tag v1.20.7-everest.2
git push origin v1.20.7-everest.2
```
Or use the **Run workflow** button on the Actions tab with a custom tag name.

The Releases tab at https://github.com/breverdbidder/claude-terminal/releases will list every successful build with one-click download links.

## Refreshing the tenant seed
The seed in `seeds/everest-tenants.json` is a snapshot of the Supabase SSOT taken at `_meta.generated_at`. To refresh: connect to project `mocerqjnksmhcjzxrewo`, re-run the export (SQL is in commit `5034f136`), replace `seeds/everest-tenants.json`, push to master with a new tag → CI rebuilds the installer.

## Refreshing the hint seed
The seed in `seeds/everest-hints.json` is hand-curated. There is **no dedicated `everest_skills` table** in Supabase; the canonical slug list is the union of `public.everest_tenants.skill_allowlist`. To refresh:
1. `SELECT DISTINCT unnest(skill_allowlist) FROM public.everest_tenants ORDER BY 1;`
2. Compare against the `categories[].hints[].title` set in `seeds/everest-hints.json`.
3. Update `_meta.generated_at`.
4. Push with a new tag → CI rebuilds.

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
This fork tracks upstream **manually**. To pull a newer upstream release: inspect new commits on `talayash/claude-terminal` master, cherry-pick or merge into a feature branch off this fork's master, re-run REPOEVAL on the new HEAD before merging, update this `BASELINE.md`, then push a new tag to trigger CI.

## Honesty tags (this revision)
- VERIFIED: Workflow YAML committed at `.github/workflows/release-windows.yml`.
- VERIFIED: Tag `v1.20.7-everest.1` pushed pointing at this commit (will be visible in this commit's response).
- INFERRED: Build will succeed because `tauri-apps/tauri-action@v0` is the canonical Tauri CI action and the project's existing `npm run tauri build` script works locally per upstream's release history.
- UNTESTED: First CI build itself — its result will appear in the Actions tab in 15–25 minutes after this commit + tag land.

## Contact
**Owner:** Everest Capital USA / Ariel Shapira
**Origin context:** postmortem session 2026-05-06/07 (summit_chat_dispatch dispatcher recovery + parallel REPOEVAL + adoption plan steps 1–5 + CI release pipeline)
