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

**P0 research-state workflow implemented.** The repository includes the Rust
package, SQLite bootstrap schema, local repository layout, target and scope
tracking, text asset import, immutable snapshots, snapshot diffs, status, and
timeline. The public command tree is direct (`githunter target ...`,
`githunter snapshot ...`) and matches the README and CLI design.

## Next milestone

P1 adds research sessions, hypotheses, tests, findings, evidence, notes, and
Markdown reports. P2 work (doctor, backups, search, completions, and large
dataset optimization) remains deliberately deferred until P1 is stable.

## Validation

On 2026-09-01, after installing stable Rust 1.98.0, the bootstrap passed:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test` (2 integration tests passed)

## Risks and decisions register

| Topic | Decision / mitigation |
|---|---|
| Scope ambiguity | Default to `UNKNOWN`; observation is never authorization. |
| Data corruption | SQLite transactions, WAL, atomic object writes, and later `doctor`. |
| Large data | Streaming imports and indexed queries; avoid full-memory loads. |
| Evidence safety | Read-only ingest, SHA-256, MIME sniffing, path containment, no auto-open. |
| BBRF overlap | Treat tool output as provenance/import data; prioritize history, snapshots, diffs, and proof chain. |
| Encoding | Existing plan displays mojibake in this terminal; new docs use plain UTF-8 ASCII-friendly Markdown. |
