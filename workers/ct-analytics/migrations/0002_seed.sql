-- One-time backfill from the legacy /stats response captured on 2026-04-27.
-- Preserves total_installations and today's distributions so the dashboard
-- doesn't reset to zero at cutover. DAU is seeded with placeholder IDs that
-- never collide with real UUIDs.

INSERT OR IGNORE INTO counters (key, value) VALUES ('total_installations', 16);

INSERT OR IGNORE INTO daily_stats (date, dimension, bucket, count) VALUES
  ('2026-04-27', 'heartbeats', '', 303),
  ('2026-04-27', 'update_checks', '', 0),
  ('2026-04-27', 'version', '1.20.4', 221),
  ('2026-04-27', 'version', '1.20.2', 69),
  ('2026-04-27', 'version', '1.20.5', 13),
  ('2026-04-27', 'os', 'windows', 303),
  ('2026-04-27', 'country', 'IL', 303);

INSERT OR IGNORE INTO daily_dau (date, installation_id) VALUES
  ('2026-04-27', '__legacy_seed_1__'),
  ('2026-04-27', '__legacy_seed_2__'),
  ('2026-04-27', '__legacy_seed_3__'),
  ('2026-04-27', '__legacy_seed_4__'),
  ('2026-04-27', '__legacy_seed_5__'),
  ('2026-04-27', '__legacy_seed_6__'),
  ('2026-04-27', '__legacy_seed_7__'),
  ('2026-04-27', '__legacy_seed_8__');
