# GITHUNTER Project Track

## Project intent

Build GITHUNTER: an offline Security Research Version Control CLI for authorized bug-bounty, pentest, lab, and CTF research. Its value is state versioning, attack-surface diffs, research provenance, evidence integrity, and report-ready history—not automated reconnaissance or exploitation.

## Activity log

### 2026-09-01 — Discovery and architecture baseline

- Read `projectPlan.md` in full.
- Inspected workspace: it contains only the specification under `ProjectTrack/`; no Rust codebase exists yet.
- Honored the specification's required first milestone: architecture/design before substantial code changes.
- Created `ARCHITECTURE.md`, `DATA_MODEL.md`, `ROADMAP.md`, `SECURITY.md`, and `CLI_DESIGN.md`.
- Defined the P0 implementation boundary and deliberately excluded automatic execution of BBRF or any other recon/scanning tool. Imports retain provenance so real-world BBRF/Burp/subfinder/httpx outputs can be tracked safely.

## Current status

**P0 & Research Extension Milestones Implemented:**
1. Core research-state workflow (init, target, scope, asset, snapshot, diff, status, timeline).
2. Bulk file-based scope rules with `#` comments, inline comments, and database-level deduplication.
3. Flexible asset ingestion (single `asset add`, file `asset import`, stdin/pipeline `cat | asset import -`).
4. Mixed asset classification & canonicalization (`DOMAIN`, `SUBDOMAIN`, `IP`, `IP_PORT`, `URL`, `ENDPOINT`).
5. Multi-source provenance tracking (multiple observations linked to a single canonical asset without duplication).
6. Configurable external tool subsystem with safe process argument vectors (opt-in only, no shell execution) and direct stdout asset ingestion.
7. Multi-step automated workflow orchestration.
8. Advisory recommendation engine (`githunter recommend`).
9. Shell completions generator for Bash, Zsh, Fish, and PowerShell (`githunter completions <shell>`).

## Next milestone

P1 adds research sessions, hypotheses, tests, findings, evidence, notes, and
Markdown reports.

## Validation

- `cargo fmt --check` (Clean formatting)
- `cargo clippy -- -D warnings` (0 warnings)
- `cargo test` (16 unit and integration tests passed)
- Release build optimized binary: `target/release/githunter`

## Risks and decisions register

| Topic | Decision / mitigation |
|---|---|
| Scope ambiguity | Default to `UNKNOWN`; observation is never authorization. |
| Data corruption | SQLite transactions, WAL, atomic object writes, and later `doctor`. |
| Large data | Streaming imports and indexed queries; avoid full-memory loads. |
| Evidence safety | Read-only ingest, SHA-256, MIME sniffing, path containment, no auto-open. |
| BBRF overlap | Treat tool output as provenance/import data; prioritize history, snapshots, diffs, and proof chain. |
| Encoding | Existing plan displays mojibake in this terminal; new docs use plain UTF-8 ASCII-friendly Markdown. |
