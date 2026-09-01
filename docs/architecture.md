# GITHUNTER Architecture

## Product boundary

GITHUNTER is an offline, local-first **Security Research Version Control CLI** for authorized work. It records research state and history. External tools can run only through an explicit user command, never through imports, recommendations, or automatic scheduling. Import provenance can name tools such as BBRF, Burp, subfinder, or httpx without invoking them.

The differentiator is a durable chain:

`scope -> observed assets -> immutable snapshot -> deterministic diff -> research session -> hypothesis -> test -> finding -> evidence -> report -> immutable timeline`

## Layers

1. **CLI** (`clap`): argument parsing, stable human/JSON output, exit-code mapping. No SQL or business rules.
2. **Application services**: use cases such as import assets, create snapshot, calculate diff, and record research activity. Transactions begin here.
3. **Domain**: validated, normalized types; scope matching; snapshot manifests; diff classification; lifecycle transitions.
4. **Repository**: project discovery, safe paths, atomic object storage, SQLite connection and migrations.
5. **Infrastructure**: SQLite, filesystem, SHA-256, MIME detection, archive implementation.

Dependencies point inward: CLI and infrastructure depend on application/domain, never the reverse.

## Proposed source layout

```text
Cargo.toml
src/
  main.rs
  cli/{mod,init,project,target,scope,asset,snapshot,diff,status,timeline}.rs
  app/{mod,project,targets,scope,assets,snapshots,diff,status,timeline}.rs
  domain/{mod,asset,scope,snapshot,diff,event,ids,validation}.rs
  db/{mod,connection,migrations,repositories}.rs
  repository/{mod,paths,objects,lock}.rs
  output/{mod,human,json}.rs
  error.rs
tests/{cli_p0.rs,fixtures/}
```

P1 modules (`research`, `hypothesis`, `test`, `finding`, `evidence`, `note`, `report`) are added only after P0 is stable.

## Repository layout

```text
<project>/.githunter/
  config.toml              # safe non-secret settings and schema version
  githunter.db                # SQLite database, WAL enabled
  objects/sha256/<prefix>/<hash>
  locks/                   # short-lived process lock metadata
  backups/
  metadata/project.json    # non-authoritative identity copy/checksum
```

User-facing `recon/`, `notes/`, `findings/`, and `evidence/` folders may be created as convenience folders. The database and object store remain authoritative; evidence is content-addressed rather than copied repeatedly.

## Operational guarantees

- Every write validates input, uses one SQLite transaction, records a timeline event, then commits.
- Snapshots and timeline events are append-only.
- Objects are written to a temporary sibling and atomically renamed after hashing.
- SQLite uses foreign keys, busy timeout, WAL mode, and short write transactions.
- Each command discovers the nearest `.githunter` ancestor; no global state or network is needed.

## Explicit saved-command pipelines

Tools may be legacy executable definitions or a saved one-line command/pipeline.
The pipeline parser understands quoting and stage separators (`|`), then starts
each executable directly with an argument array. It never invokes a shell and
rejects control operators, redirection, and command substitution syntax. Running
a tool is always an explicit command. Each execution is recorded locally with
its definition, selected input, timestamps, exit status, final stdout, and
stderr; final stdout can become provenance-bearing asset observations.

ASN and CIDR are canonical asset identities alongside host-oriented assets.
They can be exact scope rules, but are never expanded into addresses and do not
confer authorization on related assets.
