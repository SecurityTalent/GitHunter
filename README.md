<p align="center">
  <img src="src/asset/logo.png" alt="GitHunter - local-first security research workspace" width="560">
</p>

<p align="center">
  <strong>Local-first security research workspace for authorized engagements.</strong><br>
  Built by <a href="https://securitytalent.net">SecurityTalent</a>.
</p>

<p align="center">
  <a href="https://github.com/SecurityTalent/GitHunter/releases">Releases</a> ·
  <a href="#installation">Installation</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#documentation">Documentation</a> ·
  <a href="https://securitytalent.net">SecurityTalent</a>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="Apache-2.0 license"></a>
  <img src="https://img.shields.io/badge/version-1.0.0-brightgreen.svg" alt="Version 1.0.0">
  <img src="https://img.shields.io/badge/rust-1.78%2B-orange.svg" alt="Rust 1.78 or newer">
</p>

## Overview

GitHunter is an offline, local-first CLI for authorized security research. It records scope, observed assets, provenance, immutable snapshots, and changes over time in a portable `.githunter/` repository.

It is intended for authorized bug bounty programs, penetration tests, labs, CTFs, and similar approved research.

## Features

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

If Cargo's bin directory is not already on your `PATH`, add it for the current
shell session on Linux or macOS:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add the same line to your shell profile (for example, `~/.bashrc` or `~/.zshrc`)
to make it persistent.

### Build from source

```bash
git clone https://github.com/SecurityTalent/GitHunter.git
cd GitHunter
cargo build --release
```

The release binary is written to `target/release/`. On Windows, ensure
`%USERPROFILE%\.cargo\bin` is in `PATH` when using Cargo installation. For the
current PowerShell session, run:

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
```

Confirm the installation before creating a project:

```bash
githunter --version
githunter --help
```

## Before you begin

Use GitHunter only where you have clear, written authorization. Before running
an external tool, make sure that you have its permission requirements, rate
limits, and out-of-scope exclusions. GitHunter tracks and classifies assets; it
does not create authorization for a target or an external command.

External tools such as `subfinder` and `httpx` are optional and are not bundled
with GitHunter. Install and test each one separately before saving it as a
GitHunter tool.

<a id="quick-start"></a>

## Real-world walkthrough

This example creates a local project for an authorized program, defines its
scope, records initial findings, and saves a baseline. Replace `target.com`
with a target you are authorized to test.

### 1. Create a project

```bash
# Work inside a dedicated directory for this engagement.
mkdir authorized-target
cd authorized-target
githunter init
```

GitHunter creates a local `.githunter/` directory in this folder. Keep the
folder with the engagement notes so its history remains available. Running
`githunter init` starts a short setup prompt for the project name, primary
target, authorization note, in-scope domains, and out-of-scope domains. Enter
multiple scope domains separated by commas. Use `githunter init --name <name>`
for a non-interactive setup.

### 2. Record the authorized target and scope

```bash
githunter target add "target.com" --authorization "Authorized security program"
githunter scope add "target.com"
githunter scope add "*.target.com"
githunter scope out add "admin.target.com"
githunter scope list
```

An out-of-scope rule is important when a program explicitly excludes a host or
subdomain. Check a value before using it with an external tool:

```bash
githunter scope check "https://api.target.com/login"
```

Only `IN_SCOPE` is a positive scope match. `OUT_OF_SCOPE` and `UNKNOWN` must
not be treated as permission to act.

### 3. Add known assets and create a baseline

Create an `assets.txt` file with one value per line, then import it:

```bash
githunter asset import assets.txt --source manual
githunter snapshot create --note "Baseline"
githunter status
```

At this point, `status` shows the project totals and `timeline` shows the local
audit history. You now have a known starting point for later comparisons.

### 4. Add new observations and review change

```bash
githunter asset import follow-up-assets.txt --source recon
githunter snapshot create --note "Follow-up discovery"
githunter diff
githunter timeline
```

`diff` compares the two latest snapshots, making it easy to identify what
changed since the previous baseline.

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
githunter asset list --type cidr
githunter asset list --type asn
```

To import tool output from standard input, use the version for your shell:

```bash
# Linux or macOS
cat assets.txt | githunter asset import - --source pipeline
```

```powershell
# PowerShell
Get-Content assets.txt | githunter asset import - --source pipeline
```

### List and export assets

Use `asset list` to review records in GitHunter. Without filters, it shows every
tracked asset. Filter by type or source when you need a smaller view:

```bash
# Show all tracked assets.
githunter asset list

# Show only root domains or only subdomains.
githunter asset list --type domain
githunter asset list --type subdomain

# Show assets recorded by httpx.
githunter asset list --source httpx

# `all` means every scope status; `50` limits the output to 50 results.
githunter asset list --type domain all
githunter asset list --type domain all 50
```

`--type` accepts `domain`, `subdomain`, `ip`, `ip_port`, `url`, `endpoint`,
`asn`, `cidr`, or `unknown`. Scope values are `in_scope`, `out_of_scope`,
`unknown`, or `all` (for list only). Source filters match the exact source
identifier, so `--source httpx` does not include an unrelated source such as
`my-httpx`.

Use `asset export` when another tool needs clean values only, without headers
or other GitHunter output:

```bash
# Export only approved subdomains.
githunter asset export --type subdomain --scope in_scope

# Send approved subdomains to httpx. This displays httpx output only.
githunter asset export --type subdomain --scope in_scope | httpx -silent

# Send approved subdomains to httpx and save its output back into GitHunter.
githunter asset export --type subdomain --scope in_scope | httpx -silent | githunter asset import - --source httpx
```

### Interoperability with security tools

GitHunter follows standard input/output conventions so it fits into an
authorized research workflow without wrapper scripts. `asset export` writes
canonical values only—one value per line, with no headings or status text—so
its output can be consumed directly by another command. `asset import -` reads
newline-delimited output from standard input and preserves the named source as
observation provenance.

The following example discovers authorized subdomains, records them, probes
only the in-scope subdomains, then records the resulting URLs:

```bash
# External tool -> GitHunter: record passive discovery output.
subfinder -d target.example -silent \
  | githunter asset import - --source subfinder

# GitHunter -> external tool -> GitHunter: process only tracked in-scope assets.
githunter asset export --type subdomain --scope in_scope \
  | httpx -silent \
  | githunter asset import - --source httpx
```

```powershell
# The same scope-filtered probe flow in PowerShell.
githunter asset export --type subdomain --scope in_scope |
  httpx -silent |
  githunter asset import - --source httpx
```

For example, use `dnsx` to resolve only the subdomains already classified as
in scope, then retain its output as a separate source:

```bash
githunter asset export --type subdomain --scope in_scope \
  | dnsx -silent \
  | githunter asset import - --source dnsx
```

`subfinder` is useful for passive discovery, `dnsx` for DNS resolution, and
`httpx` for HTTP probing. Install each tool separately and follow the target
program's authorization and rate-limit requirements.

Before running an external tool, define the authorized target and scope rules.
GitHunter records and classifies observations; it does not grant authorization
to an external command or infer permission for `UNKNOWN` assets.

ASN values are canonicalized to forms such as `AS13335`. CIDRs are normalized to their network address, for example `192.168.1.99/24` becomes `192.168.1.0/24`.

## Using external tools

GitHunter lets you save the security tools you already use, then run them from
the project with a clear audit trail. A saved tool is only a configuration: it
never runs when you add, view, validate, or import it. Execution requires an
explicit `tool run` or `workflow run` command.

### Save, review, and run a tool

The example below saves a passive subdomain-discovery command, reviews it
without running anything, verifies that the required executable is available,
and then runs it for an authorized target. Review imported assets and their
scope status before sending them to an active probing tool.

```bash
# 1. Save a passive-discovery tool.
githunter tool add "subfinder -d {target} -silent" --name subfinder-passive

# 2. Review the command and validate its local dependencies.
githunter tool explain subfinder-passive
githunter tool validate subfinder-passive

# 3. Run it only after confirming the target is authorized.
githunter tool run subfinder-passive --target target.com

# 4. Review what was recorded before any further action.
githunter asset list --source tool:subfinder-passive
```

Use `githunter tool list` to view configured tools, `githunter tool show <name>`
for a saved configuration, and `githunter tool remove <name>` to delete one.

### Provide input safely

Use `{target}` in a saved command to pass the approved value at run time. In a
saved command or pipeline, `{asset}`, `{input}`, and `{scope}` are also
available. Choose the value source explicitly:

```bash
githunter tool run <tool-name> --target target.com
githunter tool run <tool-name> --asset api.target.com
githunter tool run <tool-name> --file targets.txt
Get-Content targets.txt | githunter tool run <tool-name> --stdin
githunter tool run <tool-name> --scope in_scope
```

When `--file`, `--stdin`, or `--scope` is used, GitHunter selects and displays
the first valid value before it starts the tool. Tool output is imported as
assets by default and recorded with its tool name as the source (for example,
`tool:subfinder-passive` for a saved command), so discoveries remain traceable.

### Pipeline safety

Pipelines may use quoted arguments and `|` separators. GitHunter executes each
stage directly instead of passing the pipeline to a shell. Shell control
operators and redirection (`;`, `&`, backticks, `<`, and `>`) are rejected.

For advanced configurations, you can define an executable and its arguments
separately:

```bash
githunter tool add --name subfinder --executable subfinder --args "-d {target} -silent"
```

### Run a repeatable workflow

After validating each saved tool, group trusted steps into a named workflow.
The steps run in the listed order and still require an explicit run command.

```bash
githunter workflow add --name daily-passive --description "Authorized passive discovery" --steps subfinder-passive
githunter workflow show daily-passive
githunter workflow run daily-passive --target target.com
```

Use `githunter workflow list` to see saved workflows and
`githunter workflow remove <name>` to remove one. Review the scope of newly
imported assets before using any active tool against them.

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

To combine the asset records from two existing snapshots without modifying
either source snapshot, create an immutable merge snapshot:

```bash
githunter snapshot merge s_0001 s_0002
```

The result contains the union of both snapshots. If the same asset has a
different historic state in both, the second snapshot argument takes priority.
Use `githunter snapshot list` to find snapshot IDs.

## Live dashboard

Open a second terminal in the project directory and run:

```bash
githunter watch
```

The dashboard refreshes the same screen every five seconds (rather than
printing a new dashboard block) and shows targets, scope rules,
asset totals by scope and type, snapshots, configured tools, and the eight most
recent local timeline events. It is read-only: it never launches `httpx` or any
other external tool. Stop it with `Ctrl+C`; use `--interval 2` for a two-second
refresh, or `--once` to print one dashboard frame for a script.

## Command reference

| Command | Purpose |
| --- | --- |
| `githunter init [--name <name>]` | Initialize a local repository. |
| `githunter target add <value>` | Register an authorized primary target. |
| `githunter target list` | List registered targets and their authorization notes. |
| `githunter scope add <pattern>` | Add an in-scope rule. |
| `githunter scope out add <pattern>` | Add an out-of-scope rule. |
| `githunter scope check <value>` | Check whether an asset matches the defined scope. |
| `githunter asset add <value>` | Record one observed asset. |
| `githunter asset import [file|-]` | Add newline-separated assets to the project. Use a file name, such as `githunter asset import assets.txt`, or use `-` to read piped input, such as `Get-Content assets.txt \| githunter asset import -`. |
| `githunter asset list [scope] [limit]` | List assets with optional type, scope, source, JSON, and result-limit filters. Use `all 50` to include every scope status and show up to 50 results. |
| `githunter asset export` | Write canonical values to stdout for another tool. |
| `githunter tool add "<command>" --name <name>` | Save a command or pipeline. |
| `githunter tool list` | List saved tool configurations. |
| `githunter tool explain <name>` | Describe stages and placeholders without execution. |
| `githunter tool validate <name>` | Check a saved configuration and its executable. |
| `githunter tool run <name>` | Explicitly run a saved tool or pipeline. |
| `githunter workflow add --name <name> --steps <tools>` | Save an ordered workflow. |
| `githunter workflow run <name>` | Explicitly run a saved workflow. |
| `githunter snapshot create` | Create an immutable snapshot. |
| `githunter snapshot merge <snapshot1> <snapshot2>` | Create an immutable union of two snapshots. |
| `githunter diff` | Compare the latest two snapshots. |
| `githunter status` | Show the project name and current totals for tracked assets and snapshots. |
| `githunter timeline` | Show the local audit timeline. |
| `githunter watch [--interval <seconds>] [--once]` | Continuously display a read-only local project dashboard. |

**Command syntax:** square brackets, such as `[file|-]`, mean the value is
optional and should not be typed. The `|` means “or”: provide a file name to
import from a file, or provide `-` to import values sent through a pipe.

## Documentation

Use `githunter --help` or `<command> --help` for complete CLI reference. Global options include `--repo <path>` and `--no-color`; design and implementation notes are available in [`docs/`](docs/).

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
