# GITHUNTER Roadmap

## P0 — research-state foundation

1. Bootstrap Rust workspace, error/output conventions, migrations, repository discovery, `init`.
2. Project metadata and `project show`.
3. Targets and explicit scope rules, including deterministic scope check.
4. Validated streaming TXT/CSV/JSON/JSONL asset import; list/show and JSON output.
5. Canonical snapshot manifests, object store, snapshot list/show.
6. Deterministic snapshot diff with type/target/since filters.
7. Research-aware status and immutable timeline.
8. Unit, integration, cross-platform path, and end-to-end tests; `fmt`, `clippy`, `test` clean.

## P1 — evidence-to-report workflow

Research sessions, hypotheses, manual test records, findings lifecycle, hashed evidence with safe MIME detection, notes, and reviewed Markdown report generation.

## P2 — operational resilience

Local search, doctor, backup/create/verify, shell completions, large-dataset benchmarks and performance work.

## P3 — deliberately deferred

Research branches, remote sync, optional integrations, optional AI. No automatic scanning, recon, exploitation, credential handling, or cloud service is in scope without a new explicit design.

## Acceptance gates

- P0 only moves forward when each command has help, human output, stable JSON where specified, meaningful exit codes, and tests.
- P1 begins only after snapshot/diff correctness, migration safety, and P0 end-to-end tests are stable.
