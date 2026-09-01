use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;

const INITIAL_SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
CREATE TABLE schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);
CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  researcher_identity TEXT,
  schema_version INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE TABLE timeline_events (
  id TEXT PRIMARY KEY,
  event_type TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  occurred_at TEXT NOT NULL,
  payload_hash TEXT NOT NULL
);
CREATE INDEX timeline_events_occurred_at ON timeline_events(occurred_at);
CREATE TABLE targets (
  id TEXT PRIMARY KEY, value TEXT NOT NULL UNIQUE, target_type TEXT NOT NULL,
  authorization_note TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE scope_rules (
  id TEXT PRIMARY KEY, pattern TEXT NOT NULL, state TEXT NOT NULL
    CHECK(state IN ('IN_SCOPE','OUT_OF_SCOPE')),
  source TEXT NOT NULL, created_at TEXT NOT NULL, UNIQUE(pattern, state)
);
CREATE TABLE assets (
  id TEXT PRIMARY KEY, asset_type TEXT NOT NULL, normalized_value TEXT NOT NULL,
  scope_status TEXT NOT NULL CHECK(scope_status IN ('IN_SCOPE','OUT_OF_SCOPE','UNKNOWN')),
  first_seen TEXT NOT NULL, last_seen TEXT NOT NULL, metadata_json TEXT NOT NULL DEFAULT '{}',
  UNIQUE(asset_type, normalized_value)
);
CREATE TABLE asset_observations (
  id TEXT PRIMARY KEY, asset_id TEXT NOT NULL REFERENCES assets(id), raw_value TEXT NOT NULL,
  source TEXT NOT NULL, observed_at TEXT NOT NULL, metadata_json TEXT NOT NULL DEFAULT '{}'
);
CREATE TABLE snapshots (
  id TEXT PRIMARY KEY, display_id TEXT NOT NULL UNIQUE, manifest_hash TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL, asset_count INTEGER NOT NULL, note TEXT
);
CREATE TABLE snapshot_assets (
  snapshot_id TEXT NOT NULL REFERENCES snapshots(id), asset_id TEXT NOT NULL REFERENCES assets(id),
  asset_hash TEXT NOT NULL, PRIMARY KEY(snapshot_id, asset_id)
);
CREATE INDEX assets_scope_status ON assets(scope_status);
CREATE INDEX assets_last_seen ON assets(last_seen);
CREATE INDEX observations_asset_observed ON asset_observations(asset_id, observed_at);
CREATE INDEX snapshot_assets_asset ON snapshot_assets(asset_id);
"#;

pub const SCHEMA_VERSION: i64 = 2;

pub fn initialize(path: &Path, project_id: &str, project_name: &str, now: &str) -> Result<()> {
    let connection = Connection::open(path)
        .with_context(|| format!("could not create local database at {}", path.display()))?;
    connection.busy_timeout(std::time::Duration::from_secs(5))?;
    connection.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")?;
    connection.execute_batch(INITIAL_SCHEMA)?;

    let transaction = connection.unchecked_transaction()?;
    transaction.execute(
        "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
        (SCHEMA_VERSION, now),
    )?;
    transaction.execute(
        "INSERT INTO projects (id, name, schema_version, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        (project_id, project_name, SCHEMA_VERSION, now),
    )?;
    transaction.commit()?;
    Ok(())
}
