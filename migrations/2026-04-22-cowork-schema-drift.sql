-- Forward-only, idempotent. Safe to re-run.
-- Brings older DBs in line with the schema files in extensions/web/schema/
-- after commit ab2f377. Apply to systemprompt-prod / FlyIO before next
-- template release.

ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS plugin_id TEXT;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS content_input_bytes BIGINT DEFAULT 0;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS content_output_bytes BIGINT DEFAULT 0;

ALTER TABLE org_marketplaces ADD COLUMN IF NOT EXISTS github_repo_url TEXT;

ALTER TABLE plugin_session_summaries ALTER COLUMN subagent_spawns TYPE BIGINT;

DROP TABLE IF EXISTS github_marketplace_sync_log;

-- New tables (employee_*, governance_decisions, org_marketplace_sync_logs)
-- are created by the schema files at next API boot via CREATE TABLE IF NOT
-- EXISTS, so no manual step needed for those.
