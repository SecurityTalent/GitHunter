# GITHUNTER Security Model

## Authorization

GITHUNTER is for authorized research. Scope is explicit and default status is `UNKNOWN`. Importing or observing an asset never grants authorization. Commands that attach active research work to `UNKNOWN` or `OUT_OF_SCOPE` assets warn clearly; no feature bypasses access controls.

## Local-data protection

- No network, shell, scanner, payload, or external-tool execution in MVP.
- Configuration rejects secret-like fields; credentials, cookies, tokens, and passwords are not persisted.
- Evidence is private/local by default; source file paths are validated, opened read-only, hashed, MIME-sniffed, and copied as data only.
- Reject absolute destination paths, traversal segments, symlinks escaping repository roots, and unsafe filenames.

## Integrity and reliability

- SHA-256 object names are recomputed on read by `doctor` and backup verification.
- Parameterized SQL, foreign keys, migrations, input-size limits, strict parsers, and allowlisted enum states are mandatory.
- Atomic filesystem writes and SQLite transactions prevent partial state. A crash may leave an unreferenced object, which `doctor` reports safely.
- Errors redact database internals and avoid leaking evidence paths unless the user requested diagnostics.

## Threats explicitly not solved in MVP

Local machine compromise, full-disk theft, and malicious files exploiting OS viewers require OS-level controls. GITHUNTER will not auto-open evidence or claim encryption at rest.
