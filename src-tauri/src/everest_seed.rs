//! Everest Capital seed: default ClaudeTerminal profiles, one per tenant.
//!
//! Source of truth: Supabase `public.everest_tenants` (project mocerqjnksmhcjzxrewo).
//! Refresh procedure: see `BASELINE.md` -> "Refreshing the tenant seed".
//!
//! This module embeds the JSON snapshot at compile time via `include_str!` so the
//! binary has no runtime network dependency. Re-running the export will require a
//! rebuild of the desktop app.

use crate::config::ConfigProfile;
use crate::database::Database;
use serde::Deserialize;
use std::collections::HashMap;

const SEED_JSON: &str = include_str!("../../seeds/everest-tenants.json");

#[derive(Debug, Deserialize)]
struct SeedFile {
    tenants: Vec<SeedTenant>,
}

#[derive(Debug, Deserialize)]
struct SeedTenant {
    slug: String,
    name: String,
    description: String,
    working_directory: String,
    claude_args: Vec<String>,
    env_vars: HashMap<String, String>,
}

/// Insert default Everest tenant profiles, but only if the `profiles` table
/// is currently empty. This guarantees user customizations are never overwritten
/// by a re-seed: once any profile exists (whether seeded or user-created),
/// this function becomes a no-op forever for that installation.
///
/// Returns `Ok(n)` where `n` is the number of profiles inserted (0 on subsequent runs).
pub fn seed_default_profiles_if_empty(db: &Database) -> Result<usize, String> {
    let existing = db.get_profiles()?;
    if !existing.is_empty() {
        return Ok(0);
    }

    let seed: SeedFile = serde_json::from_str(SEED_JSON)
        .map_err(|e| format!("Failed to parse seeds/everest-tenants.json: {}", e))?;

    let mut count = 0usize;
    for tenant in seed.tenants {
        let profile = ConfigProfile {
            id: format!("everest-{}", tenant.slug),
            name: tenant.name,
            description: Some(tenant.description),
            working_directory: tenant.working_directory,
            claude_args: tenant.claude_args,
            env_vars: tenant.env_vars,
            is_default: false,
        };
        db.save_profile(&profile)?;
        count += 1;
    }

    Ok(count)
}
