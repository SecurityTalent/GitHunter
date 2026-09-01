# 🎯 GitHunter

> **Offline, Local-First Security Research Version Control CLI**  
> Cryptographic state management, scope tracking, and attack surface diffing for authorized security research, bug bounties, pentesting, labs, and CTFs.

---

## 📖 Overview

**GitHunter** is a version-control system built specifically for security researchers. It tracks what was observed, what changed, and the provenance of your findings over time.

- 🔒 **100% Offline & Local-First:** Never makes network requests, executes shell commands, or runs automated scanners.
- 📜 **Cryptographic Snapshots:** Immutable state capture powered by SHA-256 manifests.
- 🎯 **Deterministic Scope Enforcement:** Real-time checking against explicit `IN_SCOPE` and `OUT_OF_SCOPE` wildcard rules.
- ⚡ **Deterministic Diff Engine:** Instantly identify newly introduced or removed assets between snapshots.
- 🗄️ **Robust Local Storage:** Powered by embedded SQLite (WAL mode, foreign keys) and portable `.githunter/` repositories.
- ⏱️ **Immutable Timeline & Provenance:** Full audit trail for imported outputs (e.g. Subfinder, Amass, Burp, BBRF).

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

### 2. Basic Workflow

#### Step 1: Initialize a Project
```bash
# Initialize a GitHunter repo in the current directory
githunter init --name "hackerone-target"
```

#### Step 2: Add Authorized Targets & Scope Rules
```bash
# Add authorized target root
githunter target add "target.com" --authorization "HackerOne Bug Bounty Program"

# Define In-Scope rules
githunter scope add "*.target.com"
githunter scope add "target.com"

# Define Out-of-Scope exclusions
githunter scope out add "admin.target.com"
githunter scope out add "internal.target.com"

# Check status of any asset against scope rules
githunter scope check "api.target.com"      # Outputs: IN_SCOPE  api.target.com
githunter scope check "admin.target.com"    # Outputs: OUT_OF_SCOPE  admin.target.com
githunter scope check "otherdomain.com"     # Outputs: UNKNOWN  otherdomain.com
```

#### Step 3: Ingest Discovered Assets
Import discovered subdomains, IPs, or URLs from your external recon outputs (e.g., Subfinder, httpx, Amass, Burp):

```bash
# Import assets from a text file (one per line)
githunter asset import assets.txt --source "subfinder"

# List all tracked assets and their scope classification
githunter asset list
```

#### Step 4: Create Immutable Snapshots
```bash
# Create a baseline snapshot
githunter snapshot create --note "Day 1 Recon Baseline"

# List snapshots
githunter snapshot list
```

#### Step 5: Diff Attack Surface Over Time
When you perform recon later, import new results and diff against your historical snapshots:

```bash
# Import newly observed recon data
githunter asset import new_recon.txt --source "httpx"

# Create a new snapshot
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

#### Step 6: Inspect Status & Timeline
```bash
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
| `githunter scope add <pattern>` | Add an `IN_SCOPE` wildcard or exact rule |
| `githunter scope out add <pattern>` | Add an `OUT_OF_SCOPE` exclusion rule |
| `githunter scope list` | List all defined scope rules |
| `githunter scope check <value>` | Test an asset against current scope rules |
| `githunter asset import <file> [--source <name>]` | Import domain, IP, or URL assets from file |
| `githunter asset list` | List tracked assets with scope classification |
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
├── domain/        # Pure business rules (normalization, pattern matching)
├── database/      # SQLite schema, migrations (WAL mode)
├── repository/    # Local repository discovery & lifecycle
└── main.rs        # Application entrypoint
```

### Local Repository Layout (`.githunter/`)
```text
.githunter/
├── config.toml              # Project configuration
├── githunter.db             # Embedded SQLite database
├── objects/sha256/          # Content-addressed snapshot storage
├── metadata/project.json    # Project identity
├── locks/                   # Process locking
└── backups/                 # Database snapshots & backups
```

---

## 🛡️ Security & Ethics Principles

- **Authorization First:** Observation does not imply authorization. Non-matching assets default to `UNKNOWN`.
- **Zero Recon Execution:** GitHunter never makes network requests or runs payloads; it safely stores your outputs.
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

# Execute unit and integration tests
cargo test
```

---

## 📄 License

Licensed under the [Apache-2.0 License](LICENSE).
