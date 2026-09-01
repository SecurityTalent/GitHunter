# GITHUNTER CLI, Diff, and Test Design

## Command tree

```text
githunter init [--name <name>]
githunter project show
githunter target {add,list,show,remove}
githunter scope {add,out add,list,check}
githunter asset {import,list,show}
githunter snapshot {create,list,show}
githunter diff [<from> <to>] [--type <type>] [--target <target>] [--since <snapshot>] [--json]
githunter status
githunter timeline [--target <target>] [--since <duration>]
```

P1 adds research, hypothesis, test, finding, evidence, note, and report trees. P2 adds search, doctor, backup, and completion. Global flags: `--json`, `--no-color`, and `--repo <path>`.

Exit codes: `0` success; `1` operational error; `2` invalid arguments; `3` repository missing; `4` resource missing; `5` scope violation; `6` validation error.

## Diff algorithm

1. Resolve exactly two immutable manifests: supplied IDs, or the latest two snapshots.
2. Build keyed streams/maps by immutable asset identity (`type + normalized_value`).
3. Classify keys: only-right = added; only-left = removed; both with unequal canonical asset hashes = changed; otherwise unchanged.
4. Separately compare canonical scope-rule sets and report additions/removals.
5. Apply filters before rendering; sort by type then normalized value for deterministic output.
6. Return a versioned JSON document or concise human summary. Input ordering and duplicate imports cannot affect the result.

For very large snapshots, implementation will use sorted SQLite cursors / temporary indexed tables rather than materializing every asset.

## Testing strategy

- **Unit:** host/IP/URL normalization, scope matching precedence, validation, canonical JSON/hash stability, diff classifications, safe-path checks.
- **Database:** migrations, foreign keys, transactions, deduplication, snapshot immutability, indexes/query behavior.
- **CLI integration:** missing repo, double init, target/scope flow, TXT/CSV/JSON/JSONL import accounting, list/show JSON schema, snapshot/diff/status/timeline.
- **End-to-end:** init → scope → import → snapshot → changed import → snapshot → diff → status → timeline.
- **Reliability:** invalid/traversal evidence inputs (P1), corrupt object detection (P2), interrupted writes, concurrent SQLite contention.
- **Compatibility:** Windows path cases/newlines plus Linux/macOS CI matrix when the repository exists.

Run after every module: `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`.
