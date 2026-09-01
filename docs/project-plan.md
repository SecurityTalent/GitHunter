# GITHUNTER CLI — Master Development Specification

You are a senior Rust systems engineer, CLI architect, developer-tool engineer, and security research tooling expert.

Build **GITHUNTER**, a serious, production-quality, open-source **Security Research Version Control CLI**.

## 1. Product Definition

GITHUNTER is a local-first CLI for authorized bug bounty githunters and security researchers.

The core philosophy is:

> Git versions source code.
> GITHUNTER versions security research.

GITHUNTER is NOT a Git clone.

Do not simply rename Git commands.

GITHUNTER must manage and version:

* security targets
* scope
* domains
* subdomains
* IPs
* URLs
* endpoints
* parameters
* technologies
* research sessions
* hypotheses
* tests
* findings
* evidence
* notes
* reports
* security-state snapshots
* changes
* research timeline

The primary differentiator is:

> **Security State Versioning + Attack Surface Diff + Research History**

---

# 2. Technology Requirements

Build GITHUNTER as a native CLI.

Primary language:

**Rust**

Supported operating systems:

* Windows
* Linux
* macOS

The final application should compile into a native executable.

Recommended Rust ecosystem:

* clap — CLI argument parsing
* serde / serde_json — serialization
* rusqlite or SQLx — SQLite
* anyhow / thiserror — error handling
* tracing — structured logging
* chrono or time — timestamps
* sha2 — content hashing
* uuid — IDs where appropriate
* tempfile — testing
* assert_cmd — CLI integration testing
* insta — snapshot testing where useful

Choose versions compatible with the current stable Rust toolchain.

Do not blindly add dependencies.

Every dependency must have a reason.

---

# 3. Local-First Philosophy

GITHUNTER must work completely offline.

A user must NOT need:

* an account
* a server
* PostgreSQL
* Redis
* cloud storage
* API keys

for the core functionality.

All project data should live locally.

Future remote synchronization may be added later, but it must not be required for MVP.

---

# 4. Repository Structure

A GITHUNTER project should look conceptually like:

githunter-project/
├── .githunter/
│   ├── config.toml
│   ├── githunter.db
│   ├── objects/
│   ├── snapshots/
│   ├── findings/
│   ├── evidence/
│   └── metadata/
│
├── recon/
├── notes/
├── findings/
└── evidence/

However, design the exact storage structure carefully.

Do not duplicate large datasets unnecessarily.

Use content hashes and references where appropriate.

---

# 5. Initialization

Command:

```bash
githunter init
```

Expected behavior:

1. Detect current directory.
2. Create `.githunter/`.
3. Initialize local SQLite database.
4. Create configuration.
5. Create metadata.
6. Create initial project identity.
7. Verify repository integrity.
8. Print a clear success message.

Example:

```text
Initialized GITHUNTER repository.

Project: example.com
Location: /research/example.com/.githunter

Next steps:

  githunter target add example.com
  githunter scope add ...
  githunter asset import ...
```

Running `githunter init` twice must fail safely or clearly report that the repository already exists.

---

# 6. Project

Commands:

```bash
githunter project show
githunter project config
```

Project metadata:

* project ID
* project name
* description
* created timestamp
* updated timestamp
* researcher identity
* schema version

Do not store secrets in project configuration.

---

# 7. Target Management

Commands:

```bash
githunter target add example.com
githunter target list
githunter target show example.com
githunter target remove example.com
```

Target types may include:

* domain
* wildcard domain
* IP
* URL
* application

Every target must have an explicit authorization/scope context.

Never assume that discovering an asset means it is authorized.

---

# 8. Scope Management

Commands:

```bash
githunter scope add example.com
githunter scope add '*.example.com'
githunter scope out add dev.example.com
githunter scope list
githunter scope check api.example.com
```

Scope states:

```text
IN_SCOPE
OUT_OF_SCOPE
UNKNOWN
```

Unknown assets must NOT automatically become in-scope.

The CLI should make scope status visible.

Example:

```text
$ githunter scope check api.example.com

Target: api.example.com
Scope: IN_SCOPE
Source: project scope
```

---

# 9. Asset Model

GITHUNTER should support:

```text
Domain
Subdomain
IP
URL
Endpoint
Parameter
Service
Technology
Repository
```

Every asset should contain:

* stable ID
* normalized value
* type
* scope status
* source/provenance
* first_seen
* last_seen
* tags
* metadata

Provenance examples:

```text
manual
file
subfinder
httpx
dnsx
burp
custom
```

These integrations are metadata/provenance only in MVP unless explicitly implemented later.

Do not execute external security tools automatically in the MVP.

---

# 10. Asset Import

Commands:

```bash
githunter asset import domains.txt
githunter asset import urls.txt
githunter asset import data.json
githunter asset list
githunter asset show <id>
```

Support:

* TXT
* CSV
* JSON
* JSONL

Requirements:

* normalize input
* validate input
* deduplicate
* preserve provenance
* preserve first_seen
* update last_seen
* report new vs existing items
* respect project scope

Example:

```text
Imported 1,240 assets.

New:       183
Existing:  1,047
Invalid:   10
Outscope:  0
```

---

# 11. Snapshot System

This is the HEART of GITHUNTER.

Command:

```bash
githunter snapshot create
```

A snapshot represents the security state of the project at a point in time.

It should capture references to:

* assets
* scope
* technologies
* endpoints
* parameters
* findings
* relevant research metadata

Snapshots should be immutable.

Never silently modify an existing snapshot.

Example:

```text
Snapshot: s_20260901_001
Created: 2026-09-01 12:20

Assets:
  Domains: 120
  URLs: 800
  Endpoints: 40
  Technologies: 12
```

---

# 12. Content-Addressable Storage

Use hashing where appropriate.

For large objects:

```text
SHA-256 → object ID
```

Store the object once.

Snapshots should reference objects rather than copying identical data repeatedly.

Design this carefully.

The goal is:

```text
Same data
    ↓
Same content hash
    ↓
Stored once
```

Do not implement an unnecessary or overly complicated Git internals clone.

Use Git-inspired concepts only where they make sense for security research data.

---

# 13. Snapshot List

Command:

```bash
githunter snapshot list
```

Example:

```text
ID                  DATE                  ASSETS
s_001               2026-09-01 12:20      960
s_002               2026-09-02 14:10      1,102
s_003               2026-09-05 18:42      1,390
```

Allow:

```bash
githunter snapshot show s_003
```

---

# 14. Security State Diff

Command:

```bash
githunter diff
```

or:

```bash
githunter diff s_001 s_003
```

This must be one of GITHUNTER's strongest features.

Example:

```text
GITHUNTER SECURITY DIFF
────────────────────────────────

Domains
  + api2.example.com
  + staging.example.com

URLs
  + /api/v2/users
  + /api/v2/orders

Endpoints
  + GET /api/v2/admin
  + POST /api/v2/export

Technologies
  nginx: 1.24 → 1.26

Parameters
  + user_id
  + export_format

Summary
  +25 assets
  -4 assets
  ~7 assets changed
```

Support filtering:

```bash
githunter diff --type endpoint
githunter diff --target example.com
githunter diff --since s_001
```

---

# 15. Research Sessions

Commands:

```bash
githunter research start
githunter research list
githunter research show <id>
githunter research end <id>
```

A research session represents a period of investigation.

Example:

```text
Research Session: rs_001

Target:
example.com

Started:
2026-09-01 13:10

Focus:
Authorization testing

Status:
ACTIVE
```

---

# 16. Hypothesis

Commands:

```bash
githunter hypothesis create
githunter hypothesis list
githunter hypothesis show <id>
githunter hypothesis update <id>
```

Example:

```text
Hypothesis:

Can User A access User B's object?

Status:
TESTING
```

Possible states:

```text
OPEN
TESTING
SUPPORTED
REJECTED
CONFIRMED
```

---

# 17. Tests

Commands:

```bash
githunter test create
githunter test list
githunter test show <id>
githunter test result <id>
```

A test should record:

* hypothesis
* target
* asset
* description
* timestamp
* result
* notes
* evidence references

Possible results:

```text
NOT_TESTED
FAILED
INTERESTING
CONFIRMED
INCONCLUSIVE
```

GITHUNTER should record what the researcher did.

It should NOT automatically perform intrusive exploitation.

---

# 18. Findings

Commands:

```bash
githunter finding create
githunter finding list
githunter finding show <id>
githunter finding update <id>
githunter finding close <id>
```

Finding fields:

* finding ID
* title
* severity
* CWE
* target
* asset
* endpoint
* parameter
* description
* impact
* reproduction notes
* status
* created_at
* updated_at

Severity:

```text
INFO
LOW
MEDIUM
HIGH
CRITICAL
```

Lifecycle:

```text
DISCOVERED
TESTING
CONFIRMED
REPORTING
SUBMITTED
TRIAGED
RESOLVED
CLOSED
```

---

# 19. Evidence

Commands:

```bash
githunter evidence add screenshot.png
githunter evidence add request.txt
githunter evidence add response.txt
githunter evidence list
githunter evidence show <id>
```

Evidence metadata:

* ID
* filename
* MIME type
* size
* SHA-256
* created_at
* finding reference
* test reference
* research session reference

Important:

* Never execute uploaded files.
* Never trust file extensions.
* Validate file types.
* Protect against path traversal.
* Prevent arbitrary file overwrite.
* Keep evidence private/local by default.

---

# 20. Timeline

Command:

```bash
githunter timeline
```

Example:

```text
2026-09-01 10:20  Target created
2026-09-01 10:31  183 assets imported
2026-09-01 10:45  Snapshot created
2026-09-01 11:02  Research session started
2026-09-01 11:10  Hypothesis created
2026-09-01 11:32  Test recorded
2026-09-01 11:48  Finding confirmed
2026-09-01 12:01  Evidence added
```

Timeline events must be immutable.

---

# 21. Research Log

Command:

```bash
githunter log
```

Show:

* snapshots
* research sessions
* findings
* evidence
* important changes

Support:

```bash
githunter log --finding F-001
githunter log --target example.com
githunter log --since 7d
```

---

# 22. Status

Command:

```bash
githunter status
```

Example:

```text
GITHUNTER STATUS

Project:
example.com

Current state:
Modified

Assets:
1,390

New since last snapshot:
183

Open findings:
7

Confirmed findings:
3

Active research sessions:
2

Uncommitted research changes:
YES
```

Do NOT blindly copy Git's status semantics.

Status must describe security-research state.

---

# 23. Security Research Commit Concept

GITHUNTER may support a lightweight checkpoint command:

```bash
githunter checkpoint
```

or:

```bash
githunter save
```

Do NOT call this a Git commit unless there is a strong reason.

A checkpoint should represent:

> “This is a meaningful saved state of my security research.”

Example:

```text
Saved research checkpoint.

Checkpoint:
cp_001

Changes:
+183 assets
+2 hypotheses
+1 finding
+4 evidence files
```

---

# 24. Branches

Branches are OPTIONAL for MVP.

If implemented later, they must represent research hypotheses or investigative paths, not source-code development.

Example:

```text
main
├── auth-research
├── business-logic
└── api-research
```

Do not implement branches before the core snapshot/diff system is stable.

---

# 25. Report Generation

Command:

```bash
githunter report create F-001
```

Generate a Markdown report.

Structure:

```text
Title
Severity
CWE
Target
Affected Asset
Summary
Description
Impact
Steps to Reproduce
Evidence
Remediation
Timeline
```

Do not automatically send reports anywhere.

The researcher must manually review and submit reports.

---

# 26. Search

Command:

```bash
githunter search "admin"
githunter search "api"
githunter search "authentication"
```

Search across:

* assets
* findings
* hypotheses
* notes
* timeline
* reports

Keep the implementation local and deterministic.

---

# 27. Notes

Commands:

```bash
githunter note add
githunter note list
githunter note show <id>
```

Notes can be associated with:

* target
* asset
* research session
* hypothesis
* finding

---

# 28. Output Modes

Human-readable output should be the default.

Also support machine-readable output:

```bash
githunter asset list --json
githunter snapshot show s_001 --json
githunter diff s_001 s_002 --json
```

Use stable JSON schemas.

Potential future formats:

```text
json
jsonl
csv
```

Do not break existing output contracts without versioning.

---

# 29. Exit Codes

Implement meaningful exit codes.

Example:

```text
0  Success
1  General error
2  Invalid command/arguments
3  Repository not initialized
4  Resource not found
5  Scope violation
6  Validation error
```

Document them.

---

# 30. Configuration

Use:

```text
.githunter/config.toml
```

Configuration should include only safe project settings.

Never store:

* passwords
* API keys
* cookies
* access tokens
* private credentials

unless an explicitly secure credential system is later designed.

---

# 31. Error Handling

Errors must be human-readable.

Bad:

```text
thread 'main' panicked
```

Good:

```text
Error: GITHUNTER repository not found.

Run:

    githunter init
```

Use structured internal errors with clear CLI presentation.

Never expose sensitive filesystem or database information unnecessarily.

---

# 32. Database Design

Use SQLite.

Design normalized tables for:

```text
projects
targets
scope_rules
assets
asset_observations
snapshots
snapshot_assets
changes
research_sessions
hypotheses
tests
findings
evidence
notes
timeline_events
reports
tags
```

Use foreign keys.

Use indexes for frequently searched fields.

Use migrations.

Include a schema version.

Never destroy user data during normal migrations.

---

# 33. Data Integrity

GITHUNTER must detect corruption.

Implement repository integrity checks:

```bash
githunter doctor
```

It should check:

* database integrity
* object hashes
* missing objects
* broken references
* schema version
* malformed metadata

Example:

```text
GITHUNTER DOCTOR

Database: OK
Objects: 1,293 OK
Snapshots: OK
References: OK
Evidence: 24/24 OK

Repository is healthy.
```

---

# 34. Backup

Provide:

```bash
githunter backup create
githunter backup verify
```

Backups must preserve:

* SQLite database
* objects
* snapshots
* findings
* evidence
* configuration excluding secrets

Use a safe archive format.

Never overwrite an existing backup without explicit user action.

---

# 35. Security Model

GITHUNTER is a security tool, so GITHUNTER itself must be secure.

Implement:

* strict path validation
* path traversal protection
* safe file handling
* SQLite parameterized queries
* input validation
* safe parsing
* no shell execution by default
* no arbitrary command execution
* secure temporary files
* atomic writes
* crash-safe operations
* integrity verification

External tools must NOT be automatically executed in MVP.

---

# 36. Authorization Boundary

GITHUNTER is designed for:

* bug bounty programs
* authorized penetration testing
* authorized security research
* local security labs
* CTF/lab environments

GITHUNTER must not assume authorization.

Scope must be explicit.

The tool should warn when working with:

```text
UNKNOWN
OUT_OF_SCOPE
```

Do not add features whose purpose is to bypass authorization controls.

---

# 37. Performance

The CLI must handle large datasets.

Design for:

```text
100,000+ assets
1,000,000+ URLs
large JSON/JSONL imports
many snapshots
large evidence collections
```

Do not load entire datasets into memory unnecessarily.

Use:

* streaming parsers
* batch inserts
* SQLite transactions
* indexes
* lazy loading

Benchmark important operations.

---

# 38. Concurrency

Multiple GITHUNTER processes may accidentally access the same project.

Handle SQLite locking gracefully.

Avoid database corruption.

Use transactions.

For critical writes:

```text
validate
→ transaction
→ write
→ verify
→ commit
```

---

# 39. CLI UX

Use a clean professional CLI.

Example:

```text
$ githunter status

GITHUNTER
Security Research Version Control

Project: example.com

Assets       1,390
New           183
Findings        7
Confirmed       3
Research        2
```

Avoid excessive emoji.

Avoid fake “hacker movie” styling.

Make output readable in:

* Windows Terminal
* PowerShell
* Bash
* Zsh
* CI environments

Support `--no-color`.

---

# 40. Shell Completion

Generate completion for:

* PowerShell
* Bash
* Zsh
* Fish

Command:

```bash
githunter completion
```

---

# 41. Help System

Every command must have useful help.

Examples:

```bash
githunter --help
githunter snapshot --help
githunter finding --help
githunter diff --help
```

Include examples in help output where useful.

---

# 42. Testing

Create comprehensive tests.

Unit tests:

* normalization
* hashing
* scope matching
* snapshot creation
* diff engine
* database operations
* validation

Integration tests:

* `githunter init`
* asset import
* snapshot creation
* snapshot diff
* finding creation
* evidence storage
* backup/restore

End-to-end CLI tests:

```text
init
→ target
→ scope
→ asset import
→ snapshot
→ change assets
→ second snapshot
→ diff
→ finding
→ evidence
→ report
```

Test Windows compatibility carefully.

---

# 43. Diff Engine Tests

This is critical.

Test:

```text
empty → data
data → same data
data → new asset
data → removed asset
data → changed asset
duplicate data
different ordering
scope changes
large datasets
```

The diff engine must be deterministic.

---

# 44. Architecture

Use clean modular architecture.

Recommended:

```text
src/
├── main.rs
├── cli/
│   ├── mod.rs
│   ├── init.rs
│   ├── target.rs
│   ├── scope.rs
│   ├── asset.rs
│   ├── snapshot.rs
│   ├── diff.rs
│   ├── research.rs
│   ├── hypothesis.rs
│   ├── test.rs
│   ├── finding.rs
│   ├── evidence.rs
│   ├── timeline.rs
│   ├── report.rs
│   ├── search.rs
│   ├── backup.rs
│   └── doctor.rs
│
├── domain/
├── database/
├── repository/
├── snapshot/
├── diff/
├── evidence/
├── security/
├── output/
└── errors/
```

Keep CLI parsing separate from business logic.

The core domain must be testable without invoking the CLI.

---

# 45. No Fake Features

Never create:

* fake database responses
* fake AI output
* placeholder success messages
* buttons that do nothing
* TODO implementations presented as complete

If something is not implemented, clearly mark it.

---

# 46. No Unnecessary AI

AI is NOT part of MVP.

Do not add an LLM dependency just because GITHUNTER is a security product.

First build:

```text
Repository
Asset model
Snapshot
Diff
Research history
Finding
Evidence
Timeline
```

After this is stable, an optional AI layer may be designed.

---

# 47. Future Architecture

Keep room for:

```text
githunter remote add
githunter push
githunter pull
githunter sync
```

But do NOT implement cloud synchronization in MVP.

Future integrations may include authorized:

* Burp Suite
* recon tools
* CI pipelines
* security platforms
* AI providers

Design extension points without adding unnecessary complexity now.

---

# 48. Important Product Differentiation

GITHUNTER must NOT become:

> “BBRF with different command names.”

Existing recon frameworks already manage assets such as domains, IPs, URLs and services and support scope, tagging, querying and automation.

GITHUNTER's unique value must be:

```text
Security State
      ↓
Snapshot
      ↓
Diff
      ↓
Research
      ↓
Hypothesis
      ↓
Test
      ↓
Finding
      ↓
Evidence
      ↓
Report
      ↓
Timeline
```

This complete chain is the product.

---

# 49. MVP Priority

Implement in this exact priority:

### P0

1. `githunter init`
2. SQLite repository
3. Project
4. Target
5. Scope
6. Asset model
7. Asset import
8. Asset list/show
9. Snapshot
10. Snapshot diff
11. Status
12. Timeline

### P1

13. Research sessions
14. Hypotheses
15. Tests
16. Findings
17. Evidence
18. Notes
19. Reports

### P2

20. Search
21. Doctor
22. Backup
23. Shell completion
24. Performance optimization

### P3

25. Branches
26. Remote sync
27. External integrations
28. Optional AI

Do not implement P2/P3 before P0/P1 are stable.

---

# 50. Development Process

Before writing substantial code:

## Step 1

Inspect the repository.

## Step 2

Create:

```text
ARCHITECTURE.md
DATA_MODEL.md
ROADMAP.md
SECURITY.md
```

## Step 3

Design the SQLite schema.

## Step 4

Design domain models.

## Step 5

Design CLI command hierarchy.

## Step 6

Design snapshot/diff algorithm.

## Step 7

Implement P0 incrementally.

## Step 8

Write tests after each major module.

## Step 9

Run formatting, linting and tests.

Use:

```bash
cargo fmt
cargo clippy
cargo test
```

Fix all important warnings.

## Step 10

Build release binaries for:

```text
Windows
Linux
macOS
```

---

# 51. First Coding Task

DO NOT implement the entire project immediately.

First inspect the repository and produce:

```text
1. Architecture
2. Folder structure
3. Domain model
4. SQLite schema
5. CLI command tree
6. Snapshot model
7. Diff algorithm
8. Security model
9. Testing strategy
10. MVP implementation plan
11. Risks
12. Dependencies
```

Then stop.

Do not make large code changes until the architecture is internally consistent.

After architecture approval, implement P0 one module at a time.

---

# 52. Final Product Goal

The finished tool should feel like a serious developer tool.

A researcher should be able to do:

```bash
githunter init

githunter target add example.com

githunter scope add '*.example.com'

githunter asset import subdomains.txt

githunter snapshot create

# Later...

githunter asset import new-assets.txt

githunter snapshot create

githunter diff

githunter research start

githunter hypothesis create

githunter test create

githunter finding create

githunter evidence add proof.png

githunter report create F-001

githunter timeline
```

The final experience should answer:

> What do I know?

> What changed?

> What did I investigate?

> What did I prove?

> What evidence do I have?

> What happened during my research?

That is the core of GITHUNTER.

## Final positioning

**GITHUNTER**

> Security Research Version Control CLI

> **Version your security research. Track your attack surface. Preserve your evidence.**
