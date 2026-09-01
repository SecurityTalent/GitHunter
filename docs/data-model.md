# GITHUNTER Data Model and SQLite Schema

## Identity and normalization

All records use UUID primary keys. Human IDs (`s_...`, `F-...`, `rs_...`) are generated display identifiers and have unique indexes. Asset identity is `(asset_type, normalized_value)`; imports update provenance observations and `last_seen`, never duplicate assets.

Domain normalization lowercases host names, strips a trailing dot, rejects invalid labels, and canonicalizes IP addresses. URL normalization requires HTTP(S), lowercases scheme/host, removes default ports, and preserves meaningful path/query data. Raw imported values are retained in observations for auditability.

## Core tables

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);
CREATE TABLE projects (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT,
  researcher_identity TEXT, schema_version INTEGER NOT NULL,
  created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE TABLE targets (
  id TEXT PRIMARY KEY, value TEXT NOT NULL UNIQUE, target_type TEXT NOT NULL,
  authorization_note TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE scope_rules (
  id TEXT PRIMARY KEY, pattern TEXT NOT NULL, state TEXT NOT NULL
    CHECK(state IN ('IN_SCOPE','OUT_OF_SCOPE')),
  source TEXT NOT NULL, created_at TEXT NOT NULL,
  UNIQUE(pattern, state)
);
CREATE TABLE assets (
  id TEXT PRIMARY KEY, asset_type TEXT NOT NULL,
  normalized_value TEXT NOT NULL, scope_status TEXT NOT NULL
    CHECK(scope_status IN ('IN_SCOPE','OUT_OF_SCOPE','UNKNOWN')),
  first_seen TEXT NOT NULL, last_seen TEXT NOT NULL,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  UNIQUE(asset_type, normalized_value)
);
CREATE TABLE asset_observations (
  id TEXT PRIMARY KEY, asset_id TEXT NOT NULL REFERENCES assets(id),
  raw_value TEXT NOT NULL, source TEXT NOT NULL, observed_at TEXT NOT NULL,
  metadata_json TEXT NOT NULL DEFAULT '{}'
);
CREATE TABLE tags (id TEXT PRIMARY KEY, name TEXT NOT NULL UNIQUE);
CREATE TABLE asset_tags (asset_id TEXT REFERENCES assets(id), tag_id TEXT REFERENCES tags(id),
  PRIMARY KEY(asset_id, tag_id));
CREATE TABLE snapshots (
  id TEXT PRIMARY KEY, display_id TEXT NOT NULL UNIQUE, manifest_hash TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL, asset_count INTEGER NOT NULL, note TEXT
);
CREATE TABLE snapshot_assets (
  snapshot_id TEXT NOT NULL REFERENCES snapshots(id), asset_id TEXT NOT NULL REFERENCES assets(id),
  asset_hash TEXT NOT NULL, PRIMARY KEY(snapshot_id, asset_id)
);
CREATE TABLE timeline_events (
  id TEXT PRIMARY KEY, event_type TEXT NOT NULL, entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL, occurred_at TEXT NOT NULL, payload_hash TEXT NOT NULL
);
CREATE TABLE changes (
  id TEXT PRIMARY KEY, snapshot_id TEXT NOT NULL REFERENCES snapshots(id),
  change_type TEXT NOT NULL, entity_type TEXT NOT NULL, entity_id TEXT NOT NULL,
  before_hash TEXT, after_hash TEXT
);
```

P1 adds normalized tables for research_sessions, hypotheses, tests, findings, evidence, notes, reports, and their relation tables. Evidence stores only a SHA-256 object reference, verified MIME, original basename, byte count, and associations—never an executable action.

## Required indexes

`assets(asset_type, normalized_value)`, `assets(scope_status)`, `assets(last_seen)`, `asset_observations(asset_id, observed_at)`, `snapshot_assets(asset_id)`, `timeline_events(occurred_at)`, and each P1 foreign key.

## Snapshot manifest

A canonical JSON document stores schema version, snapshot ID/time, ordered scope-rule hashes, and ordered asset records (`id`, type, normalized value, scope status, metadata hash). The SHA-256 of its UTF-8 canonical bytes is the `manifest_hash` and is stored once as an object. Snapshots reference this object and cannot be edited.
