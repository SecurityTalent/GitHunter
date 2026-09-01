# GitHunter

GitHunter is an offline, local-first command-line tool for authorized security research. It records scope, observed assets, provenance, immutable snapshots, and changes over time in a portable `.githunter/` repository.

It is intended for authorized bug bounty programs, penetration tests, labs, CTFs, and similar approved research.

## Highlights

- Local SQLite storage with WAL mode and foreign keys
- Deterministic scope rules and asset deduplication
- Asset ingestion from commands, files, and standard input
- Domain, subdomain, IP, IP:port, URL, endpoint, ASN, and CIDR support
- Immutable snapshots, deterministic diffs, status, and timeline history
- Saved tools and safe multi-stage pipelines, executed only on explicit request
- Asset provenance retained across repeated imports and tools

## Installation

### Cargo

```bash
cargo install --git https://github.com/SecurityTalent/GitHunter.git
```

### Build from source

```bash
git clone https://github.com/SecurityTalent/GitHunter.git
cd GitHunter
cargo build --release
```

The release binary is written to `target/release/`. On Windows, ensure `%USERPROFILE%\.cargo\bin` is in `PATH` when using Cargo installation.

## Quick start

```bash
githunter init --name "authorized-target"
githunter target add "target.com" --authorization "Authorized security program"
githunter scope add "target.com"
githunter scope add "*.target.com"
githunter asset import assets.txt --source manual
githunter snapshot create --note "Baseline"
githunter status
```

Check an asset before acting on it:

```bash
githunter scope check "https://api.target.com/login"
```

Possible results are `IN_SCOPE`, `OUT_OF_SCOPE`, and `UNKNOWN`. An `UNKNOWN` result is never treated as authorization.

## Asset ingestion

GitHunter automatically normalizes and deduplicates mixed assets. Files and standard input accept blank lines and `#` comments.

```text
example.com
api.example.com
8.8.8.8
10.0.0.10:443
192.168.1.99/24
AS13335
https://example.com/login
/api/v1/users
```

```bash
githunter asset add "API.EXAMPLE.COM." --source manual
githunter asset import assets.txt --source recon
cat assets.txt | githunter asset import - --source pipeline
githunter asset list --type cidr
githunter asset list --type asn
```

GitHunter can also be the source of a pipeline. `asset export` writes canonical
values only, one per line, with no table headings or status text:

```bash
githunter asset export --scope in_scope | httpx -silent
githunter asset export --type subdomain | your-tool
some-tool | githunter asset import - --source some-tool
```

ASN values are canonicalized to forms such as `AS13335`. CIDRs are normalized to their network address, for example `192.168.1.99/24` becomes `192.168.1.0/24`.

## Saved tools and pipelines

Save a command you already know rather than learning a GitHunter-specific argument format:

```bash
githunter tool add "subfinder -d {target} -silent | httpx -silent" --name passive-recon
githunter tool explain passive-recon
githunter tool validate passive-recon
githunter tool run passive-recon --target target.com
```

Tools run only when `tool run` or `workflow run` is explicitly invoked. GitHunter does not execute a saved tool when it is added, imported, recommended, or discovered.

Optional placeholders are `{target}`, `{asset}`, `{input}`, `{file}`, and `{scope}`. Select a deterministic input with one of:

```bash
githunter tool run passive-recon --target target.com
githunter tool run passive-recon --asset api.target.com
githunter tool run passive-recon --file targets.txt
cat targets.txt | githunter tool run passive-recon --stdin
githunter tool run passive-recon --scope in_scope
```

For `--file`, `--stdin`, and `--scope`, the first valid value is selected and printed before execution. Final pipeline output is ingested as assets by default and recorded with `tool:<name>` provenance.

Pipelines support quoted arguments and `|` separators. They are parsed into direct process invocations; GitHunter never passes a saved pipeline to a shell. Shell control operators and redirection (`;`, `&`, backticks, `<`, `>`) are rejected.

Legacy definitions remain available for advanced use:

```bash
githunter tool add --name subfinder --executable subfinder --args "-d {target} -silent"
```

## Scope safety

ASN and CIDR values may be stored as exact scope rules:

```bash
githunter scope add AS13335
githunter scope add 192.168.1.0/24
```

These rules are tracking rules only. GitHunter does not expand ASNs or CIDRs, scan network ranges, or infer that related assets are authorized.

## Snapshots and history

```bash
githunter snapshot create --note "Initial baseline"
githunter asset import new-assets.txt --source httpx
githunter snapshot create --note "Follow-up"
githunter diff
githunter timeline
```

Snapshots track every supported asset type. Repeated observations update provenance and `last_seen` without creating duplicate canonical assets.

## Command reference

| Command | Purpose |
| --- | --- |
| `githunter init [--name <name>]` | Initialize a local repository. |
| `githunter target add <value>` | Register an authorized primary target. |
| `githunter scope add <pattern>` | Add an in-scope rule. |
| `githunter scope out add <pattern>` | Add an out-of-scope rule. |
| `githunter asset add <value>` | Record one observed asset. |
| `githunter asset import [file|-]` | Import assets from a file or stdin. |
| `githunter asset list` | List assets with type, scope, source, or JSON filters. |
| `githunter asset export` | Write canonical values to stdout for another tool. |
| `githunter tool add "<command>" --name <name>` | Save a command or pipeline. |
| `githunter tool explain <name>` | Describe stages and placeholders without execution. |
| `githunter tool run <name>` | Explicitly run a saved tool or pipeline. |
| `githunter workflow run <name>` | Explicitly run a saved workflow. |
| `githunter snapshot create` | Create an immutable snapshot. |
| `githunter diff` | Compare the latest two snapshots. |
| `githunter status` | Show the current research-state summary. |
| `githunter timeline` | Show the local audit timeline. |

Use `githunter --help` or `<command> --help` for full options. Global options include `--repo <path>` and `--no-color`.

## Architecture

```text
src/
  cli/           Argument parsing and output
  application/   Use-case orchestration and transactions
  domain/        Validation, normalization, scope, and tool models
  database/      SQLite schema and migrations
  repository/    Local repository discovery and persistence
```

Each repository stores its data locally:

```text
.githunter/
  config.toml
  githunter.db
  metadata/project.json
  objects/sha256/
  tools/
  workflows/
  backups/
```

Further design documentation is available in [`docs/`](docs/).

## Security model

- Authorization is explicit; observation does not imply permission.
- Tool execution is explicit, local, opt-in, and auditable.
- Saved pipelines do not use shell evaluation.
- Imported values are normalized and canonical assets are deduplicated while preserving observation provenance.
- The project makes no network requests by itself.

## Development

```powershell
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## License

Licensed under the [Apache-2.0 License](LICENSE).
