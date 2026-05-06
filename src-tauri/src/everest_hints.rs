//! Everest hints SSOT — categorized skills exposed in the F1 hints panel.
//!
//! Source: hand-curated mapping over `public.everest_tenants.skill_allowlist`.
//! There is no dedicated `everest_skills` table in Supabase as of 2026-05-06;
//! the slug catalog is derived from the union of all tenants' skill_allowlist arrays.
//! Hint commands and descriptions are hand-written and embedded at compile time
//! via `include_str!`. See `BASELINE.md` -> "Refreshing the hint seed".

use crate::config::{Hint, HintCategory};
use serde::Deserialize;

const SEED_JSON: &str = include_str!("../../seeds/everest-hints.json");

#[derive(Debug, Deserialize)]
struct SeedFile {
    categories: Vec<SeedCategory>,
}

#[derive(Debug, Deserialize)]
struct SeedCategory {
    name: String,
    icon: String,
    hints: Vec<SeedHint>,
}

#[derive(Debug, Deserialize)]
struct SeedHint {
    title: String,
    command: String,
    description: String,
}

/// Load Everest skill categories from the embedded seed.
/// Returns Err if the JSON fails to parse — caller falls back to upstream defaults.
pub fn load_everest_categories() -> Result<Vec<HintCategory>, String> {
    let seed: SeedFile = serde_json::from_str(SEED_JSON)
        .map_err(|e| format!("Failed to parse seeds/everest-hints.json: {}", e))?;

    Ok(seed.categories.into_iter().map(|c| HintCategory {
        name: c.name,
        icon: c.icon,
        hints: c.hints.into_iter().map(|h| Hint {
            title: h.title,
            command: h.command,
            description: h.description,
        }).collect(),
    }).collect())
}
