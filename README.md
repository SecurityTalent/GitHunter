# GitHunter

<p align="center">
  <img src="src/asset/logo.png" alt="GitHunter logo" width="560">
</p>

GitHunter is a simple local **version-control workspace** for authorized
security research. It keeps your target, scope, discovered assets, and their
history in one folder on your computer. It does not scan anything by itself.

Use it only for a bug-bounty program, lab, CTF, penetration test, or other
work where you have permission.

## What it does

- Saves your project locally in a `.githunter` folder.
- Keeps a clear list of what is in scope and out of scope.
- Stores domains, subdomains, URLs, IPs, ASNs, and CIDRs without duplicates.
- Lets you import results from files or tools such as `subfinder` and `httpx`.
- Creates snapshots, so you can see what changed later.

## Install on Kali / WSL

Install with Cargo:

```bash
cargo install --git https://github.com/SecurityTalent/GitHunter.git --locked
githunter --help
```

If the command is not found, add Cargo to your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Build from source

```bash
git clone https://github.com/SecurityTalent/GitHunter.git
cd GitHunter
cargo build --release
./target/release/githunter --help
```

## Start here: your first project

Make one folder for one authorized target:

```bash
mkdir hackerone-test
cd hackerone-test
githunter init
```

GitHunter asks five short questions. You may press **Enter** on any question
you want to skip. The project name uses the current folder name by default.

Example:

```text
Project name [hackerone-test]: hackerone
Primary target: hackerone.com
Authorization note: HackerOne bug-bounty program
In-scope domains: hackerone.com, *.hackerone.com
Out-of-scope domains:
```

Important:

- `hackerone.com` means that exact domain.
- `*.hackerone.com` means subdomains, such as `api.hackerone.com`.
- `*hackerone.com` is also accepted and is saved as `*.hackerone.com`.
- Leave the last line blank if nothing is excluded.

If you do not want the questions, use:

```bash
githunter init --name hackerone
```

## The everyday workflow

These are the commands most people need. Replace `example.com` with a target
you are allowed to work on.

### 1. Add or review scope

```bash
# Add allowed domains.
githunter scope add example.com
githunter scope add "*.example.com"

# Add an excluded host, if the program lists one.
githunter scope out add admin.example.com

# See your rules and check one value before using an external tool.
githunter scope list
githunter scope check api.example.com
```

Only `IN_SCOPE` means the value matches a rule. `OUT_OF_SCOPE` and `UNKNOWN`
are not permission to test.

### 2. Save findings

Add one finding:

```bash
githunter asset add api.example.com --source manual
```

Or put one value per line in `assets.txt`, then import it:

```text
example.com
api.example.com
https://api.example.com/login
```

```bash
githunter asset import assets.txt --source recon
githunter asset list
```

GitHunter accepts domains, subdomains, IP addresses, IP:port values, URLs,
paths, ASNs (for example `AS13335`), and CIDRs.

### 3. Save a baseline and find changes

```bash
githunter snapshot create --note "First findings"

# Import more findings later.
githunter asset import new-assets.txt --source recon
githunter snapshot create --note "Second check"

# Show the difference between the latest two snapshots.
githunter diff
```

Useful status commands:

```bash
githunter status
githunter timeline
githunter project show
```

## Import from another tool

GitHunter does not include scanners. You install and run tools such as
`subfinder`, `dnsx`, or `httpx` yourself.

Import output from a pipe:

```bash
subfinder -d example.com -silent | githunter asset import - --source subfinder
```

Export only subdomains that GitHunter marks as in scope:

```bash
githunter asset export --type subdomain --scope in_scope
```

Example pipeline for an authorized program:

```bash
githunter asset export --type subdomain --scope in_scope \
  | httpx -silent \
  | githunter asset import - --source httpx
```

In PowerShell, use `Get-Content assets.txt | githunter asset import - --source recon`.

## Save a tool command (optional)

This is optional. Use it when you want GitHunter to remember a tool command.
Adding a tool does **not** run it.

```bash
# Save a command.
githunter tool add "subfinder -d {target} -silent" --name subfinder-passive

# Check it, then run it only for an authorized target.
githunter tool validate subfinder-passive
githunter tool run subfinder-passive --target example.com
```

`{target}` is replaced with the value you pass to `--target`. GitHunter does
not pass saved commands to a shell, and rejects shell operators such as `;`
and `>`.

## Quick command guide

| If you want to... | Use this command |
| --- | --- |
| Create a project | `githunter init` |
| See project details | `githunter project show` |
| Add a target | `githunter target add example.com` |
| Add allowed subdomains | `githunter scope add "*.example.com"` |
| Exclude a host | `githunter scope out add admin.example.com` |
| Check scope | `githunter scope check api.example.com` |
| Add one asset | `githunter asset add api.example.com` |
| Import a file | `githunter asset import assets.txt --source recon` |
| View assets | `githunter asset list` |
| View only subdomains | `githunter asset list --type subdomain` |
| Export allowed subdomains | `githunter asset export --type subdomain --scope in_scope` |
| Save a snapshot | `githunter snapshot create --note "baseline"` |
| Compare recent snapshots | `githunter diff` |
| See overall status | `githunter status` |
| Get help for any command | `githunter <command> --help` |

## Complete command and argument reference

Use this section when you need a specific option. Text in `<angle brackets>`
is required; text in `[square brackets]` is optional. Do not type the brackets.

### Global options

These can be used with every command:

```bash
# Work with a project in another folder.
githunter --repo /path/to/project status

# Disable coloured output, useful in scripts.
githunter --no-color status

# Show help or the installed version.
githunter --help
githunter --version
```

### Project, target, and scope

```bash
# Create a project without the interactive questions.
githunter init --name <project-name>

# Project details.
githunter project show

# Save a target; --authorization is optional.
githunter target add <domain-or-url> --authorization "Permission note"
githunter target list

# Add one scope rule or read one rule per line from a file.
githunter scope add <pattern>
githunter scope add --file scope.txt

# Add exclusions in the same way.
githunter scope out add <pattern>
githunter scope out add --file out-of-scope.txt

# Review or test scope.
githunter scope list
githunter scope check <domain-url-or-ip>
```

For wildcard subdomains, use `*.example.com`. A scope file may contain blank
lines and lines starting with `#`.

### Assets

```bash
# Add one asset. --source defaults to manual.
githunter asset add <value> --source <name>

# Import a file, or use - for piped input. --source defaults to file.
githunter asset import [assets.txt|-] --source <name>

# List assets. Every filter is optional.
githunter asset list \
  --type <domain|subdomain|ip|ip_port|url|endpoint|asn|cidr|unknown> \
  --scope <in_scope|out_of_scope|unknown|all> \
  --source <name> \
  --json \
  [all|in_scope|out_of_scope|unknown] [limit]

# Export clean values for another command. Every filter is optional.
githunter asset export \
  --type <type> \
  --scope <in_scope|out_of_scope|unknown> \
  --source <name>
```

`asset list` has two ways to set the scope filter: `--scope in_scope` or the
first positional value, such as `githunter asset list all 50`. Use one style,
not both. `--json` prints machine-readable JSON.

### Saved tools

There are two ways to save a tool. The first is easier; it saves a command or
a `|` pipeline. The second keeps the program and its arguments separate.

```bash
# Save a command or pipeline.
githunter tool add "subfinder -d {target} -silent" --name <tool-name>

# Save the same tool with separate options.
githunter tool add --name <tool-name> --executable subfinder \
  --args "-d {target} -silent" \
  --description "Passive discovery" \
  --input-type target \
  --output-type lines \
  --tags passive,subdomain \
  --timeout 60

# Load a complete tool definition from a TOML file.
githunter tool add --name <tool-name> --file tool.toml

# Review a saved tool; none of these commands runs it.
githunter tool list
githunter tool show <tool-name>
githunter tool explain <tool-name>
githunter tool validate <tool-name>

# Run one saved tool, or every enabled tool with all.
githunter tool run <tool-name|all> --target <target>
githunter tool run <tool-name> --asset <asset>
githunter tool run <tool-name> --file inputs.txt
printf 'api.example.com\n' | githunter tool run <tool-name> --stdin
githunter tool run <tool-name> --scope in_scope
githunter tool run <tool-name> --target <target> --no-import

# Remove a saved tool.
githunter tool remove <tool-name>
```

`--input-type` accepts `target`, `scope`, `file`, `stdin`, or `none`.
`--output-type` accepts `lines` or `json`. `--tags` takes comma-separated
names. Tool output is imported by default when a tool runs.

For a run, choose only one value source: `--target`, `--asset`, `--file`,
`--stdin`, or `--scope`. Tool stdout is recorded as assets by default; add
`--no-import` when you only want to see that tool's result.

While a saved tool runs, GitHunter shows a live `Running` line with elapsed
time. It shows `Completed`, `Failed`, or `Timed out` when the command ends.
Use `--timeout <seconds>` when saving a tool to stop a command that takes too
long. Press `Ctrl+C` to stop a running command yourself.

### Workflows

```bash
# --steps is a comma-separated list of saved tool names.
githunter workflow add --name <workflow-name> \
  --description "Daily passive checks" \
  --steps tool-one,tool-two

# Or load a complete workflow from a TOML file.
githunter workflow add --name <workflow-name> --file workflow.toml

githunter workflow list
githunter workflow show <workflow-name>
githunter workflow run <workflow-name> --target <target>
githunter workflow remove <workflow-name>
```

### Snapshots, status, and display

```bash
githunter snapshot create --note "Short description"
githunter snapshot list
githunter snapshot merge <snapshot-1> <snapshot-2>
githunter diff
githunter status
githunter timeline

# Refreshes every five seconds by default.
githunter watch --interval 10

# Show one dashboard frame and exit.
githunter watch --once

# Suggestions based on the current project state.
githunter recommend
```

### Shell completions

Generate tab-completion for your shell. Supported values are `bash`, `zsh`,
`fish`, `powershell`, and `elvish`.

```bash
githunter completions bash > githunter.bash
githunter completions zsh > _githunter
githunter completions fish > githunter.fish
```

```powershell
githunter completions powershell > githunter.ps1
```

## A few terms

| Word | Meaning |
| --- | --- |
| Target | The main authorized program or domain. |
| Scope | What the program says you may or may not test. |
| Asset | Something you found: a domain, URL, IP address, and so on. |
| Source | Where an asset came from, such as `manual`, `subfinder`, or `httpx`. |
| Snapshot | A saved picture of the assets at one point in time. |

## More commands

You do not need to memorize every option. Ask the program:

```bash
githunter --help
githunter asset --help
githunter tool --help
```

Other available commands are `workflow`, `recommend`, `watch`, `completions`,
`snapshot list`, and `timeline`.

## Safety and privacy

- All project data stays in the local `.githunter` folder.
- GitHunter itself makes no network requests.
- A discovered asset is not automatically authorized.
- External tools run only when you explicitly use `githunter tool run` or
  `githunter workflow run`.

## Development

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## License

Licensed under the [Apache-2.0 License](LICENSE).
