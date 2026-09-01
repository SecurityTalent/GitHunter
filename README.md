# 🎯 GitHunter

> **Offline, Local-First Security Research Version Control CLI**  
> Cryptographic state management, scope tracking, attack surface diffing, and configurable tool workflows for authorized security research, bug bounties, pentesting, labs, and CTFs.

---

## 📖 Overview

**GitHunter** is a version-control system built specifically for security researchers. It tracks what was observed, what changed, and the provenance of your findings over time.

- 🔒 **100% Offline & Local-First:** Never makes network requests, executes shell commands, or runs automated scanners without explicit opt-in.
- 📜 **Cryptographic Snapshots:** Immutable state capture powered by SHA-256 manifests.
- 🎯 **Deterministic Scope Enforcement & Deduplication:** Real-time checking against explicit `IN_SCOPE` and `OUT_OF_SCOPE` wildcard rules, loaded individually or via rule files with `#` comment support.
- ⚡ **Flexible Ingestion & Mixed Asset Types:** Ingest Domains, Subdomains, IPv4/IPv6, IP:Port sockets, URLs, and Endpoints from single commands, files, or stdin pipes.
- 🔍 **Multi-Source Provenance Tracking:** Deduplicates canonical assets across multiple tools (e.g. Subfinder, Amass, httpx) while preserving observation history.
- 🛠️ **Configurable Tool & Workflow Subsystem:** Explicit, opt-in execution of your favorite CLI tools with automatic stdout pipeline ingestion into GitHunter.
- 💡 **Advisory Recommendation System:** Analyzes current research state and suggests logical next steps without executing anything automatically.
- 📊 **Deterministic Diff Engine:** Instantly identify newly introduced or removed assets between snapshots.
- 🗄️ **Robust Local Storage:** Powered by embedded SQLite (WAL mode, foreign keys) and portable `.githunter/` repositories.

---

## 🚀 Installation

### ⚡ One-Command Global Install (Linux & macOS)

Run either of the following commands in your terminal to install `githunter` globally:

#### Option A: Via One-Line Install Script (Recommended)
```bash
curl -sSf https://raw.githubusercontent.com/SecurityTalent/GitHunter/main/install.sh | bash
```

#### Option B: Via Cargo (Direct from GitHub)
```bash
cargo install --git https://github.com/SecurityTalent/GitHunter.git
```

#### Option C: One-Liner Bash Command
```bash
git clone https://github.com/SecurityTalent/GitHunter.git /tmp/GitHunter && cd /tmp/GitHunter && cargo build --release && sudo install -m 755 target/release/githunter /usr/local/bin/ && cd ~ && rm -rf /tmp/GitHunter
```

*After installation, `githunter` will be accessible from anywhere in your terminal!*

---

### 🪟 Windows Installation

Open **PowerShell** and run:

```powershell
cargo install --git https://github.com/SecurityTalent/GitHunter.git
```
*(Ensure `%USERPROFILE%\.cargo\bin` is in your system PATH)*

---

### 🔨 Manual Build from Source

```bash
# Clone the repository
git clone https://github.com/SecurityTalent/GitHunter.git
cd GitHunter

# Build release binary
cargo build --release

# Copy binary to system PATH
sudo cp target/release/githunter /usr/local/bin/   # Linux / macOS
# On Windows, copy target\release\githunter.exe to your PATH folder
```

---

## ⚡ Quickstart Guide

### Step 1: Initialize a Project
```bash
# Initialize a GitHunter repository in the current directory
githunter init --name "hackerone-target"
```

### Step 2: Add Authorized Targets & Scope Rules
```bash
# Register authorized target root
githunter target add "target.com" --authorization "HackerOne Bug Bounty Program"

# Add single scope rule
githunter scope add "*.target.com"
githunter scope add "target.com"

# Or load bulk scope rules from a file (supports comments and auto-deduplication)
githunter scope add --file scope.txt
githunter scope out add --file outscope.txt

# Check scope classification of any asset
githunter scope check "api.target.com"          # Returns: IN_SCOPE
githunter scope check "admin.target.com"        # Returns: OUT_OF_SCOPE
githunter scope check "https://target.com/api"  # Returns: IN_SCOPE
githunter scope check "otherdomain.com"         # Returns: UNKNOWN
```

### Step 3: Ingest Assets (Flexible Input & Mixed Types)
GitHunter automatically normalizes and classifies mixed asset types (`DOMAIN`, `SUBDOMAIN`, `IP`, `IP_PORT`, `URL`, `ENDPOINT`):

```bash
# A) Single asset addition
githunter asset add "api.target.com"
githunter asset add "https://target.com/login" --source "burp"

# B) File import (supports mixed asset types in one file)
githunter asset import assets.txt --source "subfinder"

# C) Stdin / Pipeline ingestion
cat assets.txt | githunter asset import --source "amass"
subfinder -d target.com -silent | githunter asset import --source "subfinder"

# D) List assets with filtering or JSON output
githunter asset list --type subdomain
githunter asset list --scope in_scope
githunter asset list --json
```

### Step 4: Configure External Security Tools & Workflows (Opt-In)
Tools are defined locally and only run when explicitly triggered:

```bash
# Save a tool or pipeline exactly as you normally type it
githunter tool add "subfinder -d {target} -silent | httpx -silent" --name "passive-recon" \
  --description "Passive subdomain and HTTP discovery"

# Validate tool configuration and verify executable existence in PATH
githunter tool validate subfinder

# Run tool explicitly and auto-ingest stdout into GitHunter asset pipeline
githunter tool run passive-recon --target target.com

# Create an automated multi-step workflow
githunter workflow add --name "passive-recon" --steps "subfinder,httpx" --description "Full passive recon flow"
githunter workflow run passive-recon
```

### Step 5: Create Immutable Snapshots & Diff Over Time
```bash
# Create baseline snapshot
githunter snapshot create --note "Day 1 Recon Baseline"

# (After running new discovery or importing new scans)
githunter asset import new_recon.txt --source "httpx"
githunter snapshot create --note "Day 7 Recon Update"

# Compare changes between snapshots
githunter diff
```

Output:
```text
Diff s_0001 → s_0002
Added: 4
Removed: 0
```

### Step 6: Get Recommendations & Inspect Status
```bash
# Advisory recommendations based on current project state
githunter recommend

# Show project status overview
githunter status

# View immutable audit timeline
githunter timeline
```

---

## 🛠️ CLI Command Reference

| Command | Description |
|---|---|
| `githunter init [--name <name>]` | Initialize a new GitHunter repository (`.githunter/`) |
| `githunter project show` | Display project metadata, ID, and schema version |
| `githunter target add <val> [--authorization <note>]` | Add an authorized primary target |
| `githunter target list` | List all registered project targets |
| `githunter scope add [<pattern>] [--file <path>]` | Add an `IN_SCOPE` rule or import rule file |
| `githunter scope out add [<pattern>] [--file <path>]` | Add an `OUT_OF_SCOPE` rule or import exclusion file |
| `githunter scope list` | List all defined scope rules |
| `githunter scope check <value>` | Test an asset against current scope rules |
| `githunter asset add <value> [--source <source>]` | Ingest a single asset (domain, URL, IP, socket, endpoint) |
| `githunter asset import [<file>] [--source <source>]` | Ingest assets from a file or stdin pipe (`-`) |
| `githunter asset list [--type <t>] [--scope <s>] [--json]` | List tracked assets with filters and JSON support |
| `githunter tool list` | List all configured external tools |
| `githunter tool show <name>` | Show full configuration for a tool |
| `githunter tool add "<command>" --name <n>` | Save a single command or safe `|` pipeline; legacy `--executable` / `--args` remain supported |
| `githunter tool remove <name>` | Remove a configured tool |
| `githunter tool validate <name>` | Check tool syntax, args, and executable availability |
| `githunter tool run <name|all> [--target <t>\|--asset <a>\|--file <p>\|--stdin\|--scope <s>]` | Explicitly execute a tool/pipeline and ingest final stdout |
| `githunter tool explain <name>` | Show stages, placeholders, and safety behavior without execution |
| `githunter workflow list` | List configured automated workflows |
| `githunter workflow add --name <n> --steps <s1,s2>` | Create an ordered multi-step tool workflow |
| `githunter workflow run <name>` | Execute a workflow in deterministic sequence |
| `githunter recommend` | Advisory next-step recommendations based on state |
| `githunter completions <shell>` | Generate shell autocomplete scripts (`bash`, `zsh`, `fish`, `powershell`) |
| `githunter snapshot create [--note <note>]` | Create a cryptographic snapshot of current assets |
| `githunter snapshot list` | List all historical snapshots |
| `githunter diff` | Deterministic diff between the last two snapshots |
| `githunter status` | Summary of project targets, assets, and snapshots |
| `githunter timeline` | Chronological immutable event log |

### Global Flags
- `--repo <PATH>`: Execute command in a specific repository directory.
- `--no-color`: Disable ANSI colored terminal output.
- `-h, --help`: Display command help.
- `-V, --version`: Display tool version.

---

## 🏗️ Architecture & Storage

GitHunter is designed with strict domain separation and a layered Rust architecture:

```text
src/
├── cli/           # CLI parsing (clap) & user formatting
├── application/   # Orchestration & transactional services
├── domain/        # Pure business rules (normalization, pattern matching, tool models)
├── database/      # SQLite schema, migrations (WAL mode)
├── repository/    # Local repository discovery, tools & workflows persistence
└── main.rs        # Application entrypoint
```

### Local Repository Layout (`.githunter/`)
```text
.githunter/
├── config.toml              # Project configuration
├── githunter.db             # Embedded SQLite database (WAL mode)
├── objects/sha256/          # Content-addressed snapshot manifests
├── metadata/project.json    # Project identity
├── tools/                   # User-defined tool definitions (.toml)
├── workflows/               # User-defined workflows (.toml)
├── locks/                   # Process locking
└── backups/                 # Database snapshots & backups
```

---

## 🛡️ Security & Ethics Principles

- **Authorization First:** Observation does not imply authorization. Non-matching assets default to `UNKNOWN`.
- **Opt-In Tool Execution:** Tools are never executed implicitly or from imported asset content.
- **Safe Process Spawning:** Arguments are passed directly via arrays (`Command::args`); no shell interpreters (`sh -c`, `cmd.exe /c`) are used, preventing shell injection vulnerabilities.
- **Safe Evidence Ingestion:** All imports are normalized, hashed with SHA-256, and kept local.
- **Data Integrity:** SQLite ACID transactions, foreign keys, and WAL journal mode prevent corruption.

---

## 🧪 Development & Testing

Run the test suite and linters:

```powershell
# Format code
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Execute unit and integration tests (16 tests)
cargo test
```

---

## 📄 License

## Easy Tool Pipelines

Save commands you already know; GitHunter does not require a separate argument schema. A saved command is inert until you explicitly run it:

```powershell
githunter tool add "subfinder -d {target} -silent | httpx -silent | katana -silent" --name recon
githunter tool explain recon
githunter tool run recon --target target.com
```

`{target}`, `{asset}`, `{input}`, `{file}`, and `{scope}` are optional. `tool run` also accepts `--asset`, `--file`, `--stdin`, and `--scope in_scope`. Pipelines are parsed into direct process stages and never sent to a shell; redirection and shell-control syntax are rejected. Final stdout is ingested with `tool:<name>` provenance, while execution command, timestamps, status, stdout, and stderr remain local.

Mixed imports accept ASN (`AS13335` or `13335`, canonicalized to `AS13335`) and IPv4/IPv6 CIDR (`192.168.1.99/24` becomes `192.168.1.0/24`). They deduplicate and participate in lists, snapshots, and diffs. ASN/CIDR scope rules are exact tracking rules only: GitHunter never expands them, scans them, or infers authorization.

Licensed under the [Apache-2.0 License](LICENSE).
