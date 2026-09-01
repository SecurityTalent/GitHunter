use anyhow::{bail, Context, Result};
use clap::{Args, CommandFactory, Subcommand};
use clap_complete::Shell;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::domain::asset::{
    asset_type, classify_and_normalize, extract_matchable_host, matches_pattern, normalize,
    normalize_pattern, AssetType,
};
use crate::domain::tool::{parse_pipeline, ToolDefinition, WorkflowDefinition};
use crate::repository::Repository;

#[derive(Debug, Subcommand)]
pub enum P0Command {
    /// Display project metadata.
    Project(ProjectArgs),
    /// Manage authorized targets.
    Target(TargetArgs),
    /// Manage explicit in-scope and out-of-scope rules.
    Scope(ScopeArgs),
    /// Ingest, manage, and inspect observed assets.
    Asset(AssetArgs),
    /// Manage and inspect external security tools.
    Tool(ToolArgs),
    /// Manage and run automated tool workflows.
    Workflow(WorkflowArgs),
    /// Advisory recommendations based on project state.
    Recommend,
    /// Generate shell completion scripts.
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Create and inspect immutable security-state snapshots.
    Snapshot(SnapshotArgs),
    /// Compare security-state snapshots.
    Diff,
    /// Display the current research-state summary.
    Status,
    /// Display immutable project history.
    Timeline,
    /// Continuously display a live, read-only project dashboard.
    Watch(WatchArgs),
}

#[derive(Debug, Args)]
pub struct WatchArgs {
    /// Seconds between dashboard refreshes.
    #[arg(long, default_value_t = 5)]
    interval: u64,
    /// Render one dashboard frame and exit (useful for scripts and checks).
    #[arg(long)]
    once: bool,
}

#[derive(Debug, Args)]
pub struct ProjectArgs {
    #[command(subcommand)]
    command: ProjectCommand,
}
#[derive(Debug, Subcommand)]
enum ProjectCommand {
    Show,
}

#[derive(Debug, Args)]
pub struct TargetArgs {
    #[command(subcommand)]
    command: TargetCommand,
}
#[derive(Debug, Subcommand)]
enum TargetCommand {
    Add {
        value: String,
        #[arg(long, default_value = "Authorized project target")]
        authorization: String,
    },
    List,
}

#[derive(Debug, Args)]
pub struct ScopeArgs {
    #[command(subcommand)]
    command: ScopeCommand,
}
#[derive(Debug, Subcommand)]
pub enum ScopeCommand {
    /// Add an in-scope pattern or load rules from a file.
    Add {
        /// Pattern to add (e.g. *.target.com)
        pattern: Option<String>,
        /// Path to scope file with rules (one per line, # comments ignored)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Manage out-of-scope rules.
    Out {
        #[command(subcommand)]
        command: ScopeOutCommand,
    },
    /// List all configured scope rules.
    List,
    /// Test an asset against configured scope rules.
    Check { value: String },
}
#[derive(Debug, Subcommand)]
pub enum ScopeOutCommand {
    /// Add an out-of-scope pattern or load rules from a file.
    Add {
        /// Pattern to exclude (e.g. admin.target.com)
        pattern: Option<String>,
        /// Path to out-of-scope file with rules
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

#[derive(Debug, Args)]
pub struct AssetArgs {
    #[command(subcommand)]
    command: AssetCommand,
}
#[derive(Debug, Subcommand)]
pub enum AssetCommand {
    /// Add a single asset (domain, subdomain, IP, IP:port, URL, endpoint).
    Add {
        value: String,
        #[arg(long, default_value = "manual")]
        source: String,
    },
    /// Import assets from a file or standard input.
    Import {
        /// Path to asset file, or "-" for stdin (reads stdin if omitted and piped)
        file: Option<PathBuf>,
        /// Source identifier for provenance (e.g. subfinder, amass, httpx)
        #[arg(long, default_value = "file")]
        source: String,
    },
    /// List tracked assets with optional filtering.
    List {
        /// Filter by asset type (DOMAIN, SUBDOMAIN, IP, IP_PORT, URL, ENDPOINT, ASN, CIDR)
        #[arg(long = "type")]
        asset_type: Option<String>,
        /// Filter by scope status (IN_SCOPE, OUT_OF_SCOPE, UNKNOWN)
        #[arg(long)]
        scope: Option<String>,
        /// Filter by observation source
        #[arg(long)]
        source: Option<String>,
        /// Output in JSON format
        #[arg(long)]
        json: bool,
        /// Optional scope selector: all, in_scope, out_of_scope, or unknown.
        scope_selector: Option<String>,
        /// Maximum number of results to display.
        limit: Option<usize>,
    },
    /// Write canonical asset values to stdout, one per line, for Unix pipelines.
    Export {
        /// Filter by asset type.
        #[arg(long = "type")]
        asset_type: Option<String>,
        /// Filter by scope status (IN_SCOPE, OUT_OF_SCOPE, UNKNOWN).
        #[arg(long)]
        scope: Option<String>,
        /// Filter by observation source.
        #[arg(long)]
        source: Option<String>,
    },
}

#[derive(Debug, Args)]
pub struct ToolArgs {
    #[command(subcommand)]
    command: ToolCommand,
}
#[derive(Debug, Subcommand)]
pub enum ToolCommand {
    /// List all configured external tools.
    List,
    /// Show details of a specific tool.
    Show { name: String },
    /// Explain a saved command without executing it.
    Explain { name: String },
    /// Add or configure an external tool.
    Add {
        /// A familiar command or `|` pipeline (no shell operators are allowed).
        command: Option<String>,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "")]
        executable: String,
        #[arg(long, default_value = "")]
        description: String,
        #[arg(long, value_delimiter = ' ')]
        args: Vec<String>,
        #[arg(long, default_value = "target")]
        input_type: String,
        #[arg(long, default_value = "lines")]
        output_type: String,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        #[arg(long)]
        timeout: Option<u64>,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Remove a configured tool.
    Remove { name: String },
    /// Validate tool configuration and check executable availability.
    Validate { name: String },
    /// Explicitly execute a tool (opt-in) and optionally ingest output.
    Run {
        /// Tool name to run, or "all" to run all enabled tools
        name: String,
        /// Override target for the tool
        #[arg(long)]
        target: Option<String>,
        /// Use one explicit asset as the placeholder input.
        #[arg(long)]
        asset: Option<String>,
        /// Read the first non-comment value from a local input file.
        #[arg(long)]
        file: Option<PathBuf>,
        /// Read the first non-comment value from standard input.
        #[arg(long)]
        stdin: bool,
        /// Select the first project asset with this scope status (e.g. in_scope).
        #[arg(long)]
        scope: Option<String>,
        /// Ingest tool stdout directly into GitHunter assets
        #[arg(long, default_value_t = true)]
        import: bool,
    },
}

#[derive(Debug, Args)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    command: WorkflowCommand,
}
#[derive(Debug, Subcommand)]
pub enum WorkflowCommand {
    /// List all configured workflows.
    List,
    /// Show details of a specific workflow.
    Show { name: String },
    /// Add a new workflow with ordered tool steps.
    Add {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "")]
        description: String,
        #[arg(long, value_delimiter = ',')]
        steps: Vec<String>,
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Remove a workflow.
    Remove { name: String },
    /// Run a workflow deterministically.
    Run {
        name: String,
        #[arg(long)]
        target: Option<String>,
    },
}

#[derive(Debug, Args)]
pub struct SnapshotArgs {
    #[command(subcommand)]
    command: SnapshotCommand,
}
#[derive(Debug, Subcommand)]
enum SnapshotCommand {
    Create {
        #[arg(long)]
        note: Option<String>,
    },
    List,
    /// Create an immutable union of two snapshots; the second snapshot wins asset-state ties.
    Merge {
        snapshot1: String,
        snapshot2: String,
    },
}

pub fn execute(path: Option<PathBuf>, command: P0Command) -> Result<()> {
    if let P0Command::Completions { shell } = command {
        let mut cmd = crate::cli::Cli::command();
        clap_complete::generate(shell, &mut cmd, "githunter", &mut std::io::stdout());
        return Ok(());
    }

    let root = path
        .unwrap_or(std::env::current_dir().context("could not determine the current directory")?);
    let repo_dir = Repository::discover(&root)?;
    let mut db = Repository::open(&root)?;

    match command {
        P0Command::Project(args) => match args.command {
            ProjectCommand::Show => project_show(&db),
        },
        P0Command::Target(args) => match args.command {
            TargetCommand::Add {
                value,
                authorization,
            } => target_add(&mut db, &value, &authorization),
            TargetCommand::List => target_list(&db),
        },
        P0Command::Scope(args) => match args.command {
            ScopeCommand::Add { pattern, file } => {
                scope_add_dispatch(&mut db, pattern.as_deref(), file.as_deref(), "IN_SCOPE")
            }
            ScopeCommand::Out {
                command: ScopeOutCommand::Add { pattern, file },
            } => scope_add_dispatch(&mut db, pattern.as_deref(), file.as_deref(), "OUT_OF_SCOPE"),
            ScopeCommand::List => scope_list(&db),
            ScopeCommand::Check { value } => {
                let status = scope_status(&db, &value)?;
                println!("{status:<12} {value}");
                Ok(())
            }
        },
        P0Command::Asset(args) => match args.command {
            AssetCommand::Add { value, source } => asset_add(&mut db, &value, &source),
            AssetCommand::Import { file, source } => {
                asset_import(&mut db, file.as_deref(), &source)
            }
            AssetCommand::List {
                asset_type,
                scope,
                source,
                json,
                scope_selector,
                limit,
            } => asset_list(
                &db,
                asset_type.as_deref(),
                resolve_list_scope(scope, scope_selector)?.as_deref(),
                source.as_deref(),
                json,
                limit,
            ),
            AssetCommand::Export {
                asset_type,
                scope,
                source,
            } => asset_export(
                &db,
                asset_type.as_deref(),
                scope.as_deref(),
                source.as_deref(),
            ),
        },
        P0Command::Tool(args) => match args.command {
            ToolCommand::List => tool_list(&repo_dir, &db),
            ToolCommand::Show { name } => tool_show(&repo_dir, &db, &name),
            ToolCommand::Explain { name } => tool_explain(&repo_dir, &db, &name),
            ToolCommand::Add {
                command,
                name,
                executable,
                description,
                args,
                input_type,
                output_type,
                tags,
                timeout,
                file,
            } => {
                let tool = if let Some(file_path) = file {
                    let content = fs::read_to_string(&file_path)?;
                    toml::from_str::<ToolDefinition>(&content)?
                } else {
                    let pipeline = command.as_deref().map(parse_pipeline).transpose()?;
                    let (executable, arguments) = if let Some(stages) = pipeline {
                        (stages[0][0].clone(), stages[0][1..].to_vec())
                    } else {
                        (executable, args)
                    };
                    ToolDefinition {
                        name,
                        description,
                        executable,
                        arguments,
                        input_type,
                        output_type,
                        enabled: true,
                        timeout_seconds: timeout,
                        tags,
                        command: command.unwrap_or_default(),
                    }
                };
                tool_add(&repo_dir, &mut db, tool)
            }
            ToolCommand::Remove { name } => tool_remove(&repo_dir, &mut db, &name),
            ToolCommand::Validate { name } => tool_validate(&repo_dir, &db, &name),
            ToolCommand::Run {
                name,
                target,
                asset,
                file,
                stdin,
                scope,
                import,
            } => {
                let selected_target = resolve_run_value(
                    &db,
                    target.or(asset),
                    file.as_deref(),
                    stdin,
                    scope.as_deref(),
                )?;
                if name == "all" {
                    tool_run_all(&repo_dir, &mut db, Some(&selected_target), import)
                } else {
                    tool_run(&repo_dir, &mut db, &name, Some(&selected_target), import)
                }
            }
        },
        P0Command::Workflow(args) => match args.command {
            WorkflowCommand::List => workflow_list(&repo_dir, &db),
            WorkflowCommand::Show { name } => workflow_show(&repo_dir, &db, &name),
            WorkflowCommand::Add {
                name,
                description,
                steps,
                file,
            } => {
                let wf = if let Some(file_path) = file {
                    let content = fs::read_to_string(&file_path)?;
                    toml::from_str::<WorkflowDefinition>(&content)?
                } else {
                    WorkflowDefinition {
                        name,
                        description,
                        steps,
                    }
                };
                workflow_add(&repo_dir, &mut db, wf)
            }
            WorkflowCommand::Remove { name } => workflow_remove(&repo_dir, &mut db, &name),
            WorkflowCommand::Run { name, target } => {
                workflow_run(&repo_dir, &mut db, &name, target.as_deref())
            }
        },
        P0Command::Recommend => recommend(&repo_dir, &db),
        P0Command::Completions { .. } => unreachable!(),
        P0Command::Snapshot(args) => match args.command {
            SnapshotCommand::Create { note } => snapshot_create(&mut db, note.as_deref()),
            SnapshotCommand::List => snapshot_list(&db),
            SnapshotCommand::Merge {
                snapshot1,
                snapshot2,
            } => snapshot_merge(&mut db, &snapshot1, &snapshot2),
        },
        P0Command::Diff => diff(&db),
        P0Command::Status => status(&db),
        P0Command::Timeline => timeline(&db),
        P0Command::Watch(args) => watch(&repo_dir, &db, args.interval, args.once),
    }
}

fn now() -> Result<String> {
    Ok(OffsetDateTime::now_utc().format(&Rfc3339)?)
}

fn resolve_run_value(
    db: &Connection,
    explicit: Option<String>,
    file: Option<&Path>,
    stdin: bool,
    scope: Option<&str>,
) -> Result<String> {
    if let Some(value) = explicit {
        return Ok(value);
    }
    if let Some(path) = file {
        return fs::read_to_string(path)
            .with_context(|| format!("could not read input file {}", path.display()))?
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_owned)
            .context("input file contains no usable value");
    }
    if stdin {
        let mut value = String::new();
        io::stdin().read_to_string(&mut value)?;
        return value
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && !line.starts_with('#'))
            .map(str::to_owned)
            .context("stdin contains no usable value");
    }
    if let Some(state) = scope {
        return db.query_row("SELECT normalized_value FROM assets WHERE scope_status=?1 ORDER BY normalized_value LIMIT 1", [state.to_ascii_uppercase()], |r| r.get(0)).optional()?
            .context("no project asset matched --scope");
    }
    db.query_row(
        "SELECT value FROM targets ORDER BY value LIMIT 1",
        [],
        |r| r.get(0),
    )
    .optional()?
    .context(
        "no target found. Add a target or specify --target, --asset, --file, --stdin, or --scope",
    )
}

fn event(db: &Connection, kind: &str, entity: &str, id: &str) -> Result<()> {
    db.execute(
        "INSERT INTO timeline_events VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            Uuid::new_v4().to_string(),
            kind,
            entity,
            id,
            now()?,
            "local"
        ],
    )?;
    Ok(())
}

fn project_show(db: &Connection) -> Result<()> {
    let (name, id, created): (String, String, String) =
        db.query_row("SELECT name,id,created_at FROM projects LIMIT 1", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
    let schema: i64 = db.query_row("SELECT schema_version FROM projects LIMIT 1", [], |r| {
        r.get(0)
    })?;
    println!("Project: {name}\nID: {id}\nCreated: {created}\nSchema: {schema}");
    Ok(())
}

fn target_add(db: &mut Connection, value: &str, authorization: &str) -> Result<()> {
    let v = normalize(value)?;
    let id = Uuid::new_v4().to_string();
    db.execute(
        "INSERT INTO targets VALUES (?1,?2,?3,?4,?5)",
        params![id, v, asset_type(&v), authorization, now()?],
    )
    .context("target already exists or is invalid")?;
    event(db, "target.created", "target", &id)?;
    println!("Added target: {v}");
    Ok(())
}

fn target_list(db: &Connection) -> Result<()> {
    let mut s =
        db.prepare("SELECT value,target_type,authorization_note FROM targets ORDER BY value")?;
    let rows = s.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    for row in rows {
        let (v, t, a) = row?;
        println!("{t:<8} {v} — {a}");
    }
    Ok(())
}

fn scope_add_dispatch(
    db: &mut Connection,
    pattern: Option<&str>,
    file: Option<&Path>,
    state: &str,
) -> Result<()> {
    let mut lines_to_process = Vec::new();
    let mut is_file_source = false;

    if let Some(file_path) = file {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("could not read scope file {}", file_path.display()))?;
        lines_to_process.extend(content.lines().map(|s| s.to_string()));
        is_file_source = true;
    } else if let Some(pat) = pattern {
        let path = Path::new(pat);
        if path.is_file() && !pat.starts_with('*') {
            // Positional file detection fallback
            let content = fs::read_to_string(path)
                .with_context(|| format!("could not read scope file {}", path.display()))?;
            lines_to_process.extend(content.lines().map(|s| s.to_string()));
            is_file_source = true;
        } else {
            lines_to_process.push(pat.to_string());
        }
    } else {
        bail!("scope pattern or --file <PATH> is required");
    }

    let mut added = 0;
    let mut duplicates = 0;
    let mut last_added_pattern = String::new();

    let tx = db.transaction()?;
    for raw in lines_to_process {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Extract inline comment if any
        let clean_line = match trimmed.find('#') {
            Some(pos) => trimmed[..pos].trim(),
            None => trimmed,
        };
        if clean_line.is_empty() {
            continue;
        }

        let p = match normalize_pattern(clean_line) {
            Ok(p) => p,
            Err(e) => {
                bail!("invalid scope rule '{clean_line}': {e}");
            }
        };

        // Check deduplication at database level
        let count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM scope_rules WHERE pattern=?1 AND state=?2",
            params![p, state],
            |r| r.get(0),
        )?;

        last_added_pattern = p.clone();
        if count > 0 {
            duplicates += 1;
        } else {
            let id = Uuid::new_v4().to_string();
            let seen = now()?;
            tx.execute(
                "INSERT INTO scope_rules VALUES (?1,?2,?3,?4,?5)",
                params![id, p, state, "project scope", seen],
            )?;
            added += 1;
        }
    }
    tx.commit()?;

    if added > 0 {
        event(db, "scope.created", "scope_rule", state)?;
    }

    if is_file_source {
        println!("Added: {added}");
        println!("Skipped duplicates: {duplicates}");
    } else if added > 0 {
        println!("Added {state}: {last_added_pattern}");
    } else {
        println!("Skipped duplicate: {last_added_pattern}");
    }

    Ok(())
}

fn scope_list(db: &Connection) -> Result<()> {
    let mut s = db.prepare("SELECT state,pattern FROM scope_rules ORDER BY state,pattern")?;
    for row in s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))? {
        let (a, b) = row?;
        println!("{a:<12} {b}");
    }
    Ok(())
}

pub fn scope_status(db: &Connection, value: &str) -> Result<String> {
    let (asset_type, norm_val) = match classify_and_normalize(value) {
        Ok(res) => res,
        Err(_) => return Ok("UNKNOWN".into()),
    };
    let host_opt = extract_matchable_host(asset_type, &norm_val);
    let match_target = host_opt.as_deref().unwrap_or(&norm_val);

    let mut s = db.prepare("SELECT pattern,state FROM scope_rules")?;
    let rules = s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut inside = false;
    for rule in rules {
        let (p, state) = rule?;
        if matches_pattern(&p, match_target) {
            if state == "OUT_OF_SCOPE" {
                return Ok("OUT_OF_SCOPE".into());
            }
            inside = true;
        }
    }
    Ok(if inside {
        "IN_SCOPE".into()
    } else {
        "UNKNOWN".into()
    })
}

fn asset_add(db: &mut Connection, value: &str, source: &str) -> Result<()> {
    let (kind, canonical) = classify_and_normalize(value)
        .with_context(|| format!("could not parse asset value '{value}'"))?;
    let seen = now()?;
    let status = scope_status(db, &canonical)?;

    let existing_id: Option<String> = db
        .query_row(
            "SELECT id FROM assets WHERE asset_type=?1 AND normalized_value=?2",
            params![kind.as_str(), canonical],
            |r| r.get(0),
        )
        .optional()?;

    let (asset_id, is_new) = match existing_id {
        Some(id) => {
            db.execute(
                "UPDATE assets SET last_seen=?1, scope_status=?2 WHERE id=?3",
                params![seen, status, id],
            )?;
            (id, false)
        }
        None => {
            let id = Uuid::new_v4().to_string();
            db.execute(
                "INSERT INTO assets VALUES (?1,?2,?3,?4,?5,?5,'{}')",
                params![id, kind.as_str(), canonical, status, seen],
            )?;
            (id, true)
        }
    };

    db.execute(
        "INSERT INTO asset_observations VALUES (?1,?2,?3,?4,?5,'{}')",
        params![Uuid::new_v4().to_string(), asset_id, value, source, seen],
    )?;

    event(db, "asset.added", "asset", &asset_id)?;

    if is_new {
        println!(
            "Added asset: {canonical} ({}, {status})",
            kind.display_label()
        );
    } else {
        println!(
            "Asset already tracked: {canonical} ({}, {status}). Recorded observation from source '{source}'.",
            kind.display_label()
        );
    }
    Ok(())
}

fn asset_import(db: &mut Connection, file: Option<&Path>, source: &str) -> Result<()> {
    if let Some(path) = file {
        if path == Path::new("-") {
            let stdin = io::stdin();
            let reader = stdin.lock();
            import_assets_from_reader(db, reader, source)?;
        } else {
            let f = fs::File::open(path)
                .with_context(|| format!("could not open asset file {}", path.display()))?;
            let reader = io::BufReader::new(f);
            import_assets_from_reader(db, reader, source)?;
        }
    } else {
        let stdin = io::stdin();
        if stdin.is_terminal() {
            bail!("missing asset input file or piped stdin. Use `githunter asset import <file>` or pipe data via stdin.");
        }
        let reader = stdin.lock();
        import_assets_from_reader(db, reader, source)?;
    }
    Ok(())
}

pub fn import_assets_from_reader<R: BufRead>(
    db: &mut Connection,
    reader: R,
    source: &str,
) -> Result<()> {
    let tx = db.transaction()?;
    let mut added = 0;
    let mut existing = 0;
    let mut invalid = 0;

    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut scope_counts: HashMap<String, usize> = HashMap::new();

    for line_res in reader.lines() {
        let raw = line_res?;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let (kind, canonical) = match classify_and_normalize(trimmed) {
            Ok(v) => v,
            Err(_) => {
                invalid += 1;
                continue;
            }
        };

        let kind_str = kind.as_str().to_string();
        let display_type = kind.display_label().to_string();
        let seen = now()?;
        let status = scope_status(&tx, &canonical)?;

        *type_counts.entry(display_type).or_insert(0) += 1;
        *scope_counts.entry(status.clone()).or_insert(0) += 1;

        let id: Option<String> = tx
            .query_row(
                "SELECT id FROM assets WHERE asset_type=?1 AND normalized_value=?2",
                params![kind_str, canonical],
                |r| r.get(0),
            )
            .optional()?;

        let asset_id = match id {
            Some(id) => {
                existing += 1;
                tx.execute(
                    "UPDATE assets SET last_seen=?1, scope_status=?2 WHERE id=?3",
                    params![seen, status, id],
                )?;
                id
            }
            None => {
                added += 1;
                let id = Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO assets VALUES (?1,?2,?3,?4,?5,?5,'{}')",
                    params![id, kind_str, canonical, status, seen],
                )?;
                id
            }
        };

        tx.execute(
            "INSERT INTO asset_observations VALUES (?1,?2,?3,?4,?5,'{}')",
            params![Uuid::new_v4().to_string(), asset_id, trimmed, source, seen],
        )?;
    }

    tx.commit()?;

    if added > 0 || existing > 0 {
        event(db, "assets.imported", "asset_import", source)?;
    }

    let total = added + existing;
    println!("Imported: {total}");
    println!("New assets: {added}");
    println!("Duplicates: {existing}");
    if invalid > 0 {
        println!("Invalid lines: {invalid}");
    }
    println!();
    println!("Types:");
    for t in [
        "DOMAIN",
        "SUBDOMAIN",
        "IP",
        "IP_PORT",
        "URL",
        "ENDPOINT",
        "ASN",
        "CIDR",
        "UNKNOWN",
    ] {
        if let Some(count) = type_counts.get(t) {
            println!("  {t:<10} {count}");
        }
    }
    println!();
    println!("Scope:");
    for s in ["IN_SCOPE", "OUT_OF_SCOPE", "UNKNOWN"] {
        let count = scope_counts.get(s).copied().unwrap_or(0);
        println!("  {s:<12} {count}");
    }

    Ok(())
}

fn resolve_list_scope(
    long_scope: Option<String>,
    positional_scope: Option<String>,
) -> Result<Option<String>> {
    if long_scope.is_some() && positional_scope.is_some() {
        bail!("use either --scope <status> or the positional scope selector, not both");
    }

    let Some(scope) = long_scope.or(positional_scope) else {
        return Ok(None);
    };
    let normalized = scope.to_ascii_uppercase();
    match normalized.as_str() {
        "ALL" => Ok(None),
        "IN_SCOPE" | "OUT_OF_SCOPE" | "UNKNOWN" => Ok(Some(normalized)),
        _ => bail!("invalid scope '{scope}'; use all, in_scope, out_of_scope, or unknown"),
    }
}

fn resolve_asset_type(asset_type: &str) -> Result<String> {
    AssetType::parse(asset_type)
        .map(|kind| kind.as_str().to_owned())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "invalid asset type '{asset_type}'; use domain, subdomain, ip, ip_port, url, endpoint, asn, cidr, or unknown"
            )
        })
}

fn resolve_export_scope(scope: &str) -> Result<String> {
    resolve_list_scope(Some(scope.to_owned()), None)?.ok_or_else(|| {
        anyhow::anyhow!(
            "asset export does not accept scope 'all'; omit --scope to export every scope status"
        )
    })
}

fn asset_list(
    db: &Connection,
    type_filter: Option<&str>,
    scope_filter: Option<&str>,
    source_filter: Option<&str>,
    json_output: bool,
    limit: Option<usize>,
) -> Result<()> {
    let mut query = String::from(
        "SELECT a.id, a.asset_type, a.normalized_value, a.scope_status, a.first_seen, a.last_seen,
                GROUP_CONCAT(DISTINCT o.source) AS sources
         FROM assets a
         LEFT JOIN asset_observations o ON a.id = o.asset_id
         WHERE 1=1 ",
    );

    let mut param_values: Vec<String> = Vec::new();

    if let Some(t) = type_filter {
        query.push_str("AND a.asset_type = ? ");
        param_values.push(resolve_asset_type(t)?);
    }

    if let Some(s) = scope_filter {
        query.push_str("AND a.scope_status = ? ");
        param_values.push(s.to_ascii_uppercase());
    }

    if let Some(src) = source_filter {
        query.push_str(
            "AND EXISTS (SELECT 1 FROM asset_observations source_observation WHERE source_observation.asset_id=a.id AND source_observation.source=?) ",
        );
        param_values.push(src.to_owned());
    }

    query.push_str("GROUP BY a.id ");

    query.push_str("ORDER BY a.asset_type, a.normalized_value ");

    if let Some(max_results) = limit {
        query.push_str("LIMIT ? ");
        param_values.push(max_results.to_string());
    }

    let mut stmt = db.prepare(&query)?;
    let params_ref: Vec<&dyn rusqlite::ToSql> = param_values
        .iter()
        .map(|v| v as &dyn rusqlite::ToSql)
        .collect();

    let rows = stmt.query_map(params_ref.as_slice(), |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, Option<String>>(6)?.unwrap_or_default(),
        ))
    })?;

    if json_output {
        let mut list = Vec::new();
        for row in rows {
            let (id, kind, val, scope, first_seen, last_seen, sources_str) = row?;
            let sources: Vec<&str> = sources_str.split(',').filter(|s| !s.is_empty()).collect();
            list.push(json!({
                "id": id,
                "type": kind,
                "value": val,
                "scope": scope,
                "first_seen": first_seen,
                "last_seen": last_seen,
                "sources": sources
            }));
        }
        println!("{}", serde_json::to_string_pretty(&list)?);
    } else {
        println!(
            "{:<12} {:<10} {:<36} {:<20} {:<24}",
            "SCOPE", "TYPE", "VALUE", "SOURCES", "LAST SEEN"
        );
        println!("{}", "-".repeat(105));
        for row in rows {
            let (_, kind, val, scope, _, last_seen, sources) = row?;
            println!("{scope:<12} {kind:<10} {val:<36} {sources:<20} {last_seen:<24}");
        }
    }
    Ok(())
}

/// Emits values only (no headings or status text), making it safe to pipe into
/// other programs without requiring text parsing.
fn asset_export(
    db: &Connection,
    type_filter: Option<&str>,
    scope_filter: Option<&str>,
    source_filter: Option<&str>,
) -> Result<()> {
    let mut query = String::from("SELECT DISTINCT a.normalized_value FROM assets a WHERE 1=1 ");
    let mut values: Vec<String> = Vec::new();
    if let Some(kind) = type_filter {
        query.push_str("AND a.asset_type=? ");
        values.push(resolve_asset_type(kind)?);
    }
    if let Some(scope) = scope_filter {
        query.push_str("AND a.scope_status=? ");
        values.push(resolve_export_scope(scope)?);
    }
    if let Some(source) = source_filter {
        query.push_str(
            "AND EXISTS (SELECT 1 FROM asset_observations o WHERE o.asset_id=a.id AND o.source=?) ",
        );
        values.push(source.to_owned());
    }
    query.push_str("ORDER BY a.normalized_value");
    let params: Vec<&dyn rusqlite::ToSql> =
        values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
    let mut statement = db.prepare(&query)?;
    let rows = statement.query_map(params.as_slice(), |row| row.get::<_, String>(0))?;
    for row in rows {
        println!("{}", row?);
    }
    Ok(())
}

fn tool_list(repo_dir: &Path, _db: &Connection) -> Result<()> {
    let tools = Repository::load_tool_files(repo_dir)?;
    if tools.is_empty() {
        println!("No external tools configured.");
        println!("Add tools with `githunter tool add \"<command>\" --name <name>`");
        return Ok(());
    }

    println!(
        "{:<16} {:<8} {:<28} {:<30}",
        "NAME", "ENABLED", "TAGS / CATEGORY", "DESCRIPTION"
    );
    println!("{}", "-".repeat(84));
    for tool in tools {
        let enabled_str = if tool.enabled { "yes" } else { "no" };
        let tags_str = tool.tags.join(", ");
        println!(
            "{:<16} {:<8} {:<28} {:<30}",
            tool.name, enabled_str, tags_str, tool.description
        );
    }
    Ok(())
}

fn tool_show(repo_dir: &Path, _db: &Connection, name: &str) -> Result<()> {
    let tools = Repository::load_tool_files(repo_dir)?;
    let tool = tools
        .into_iter()
        .find(|t| t.name == name)
        .with_context(|| format!("tool '{name}' not found"))?;

    println!("Tool: {}", tool.name);
    println!("Description: {}", tool.description);
    println!("Executable: {}", tool.executable);
    println!("Arguments: {:?}", tool.arguments);
    println!("Input Type: {}", tool.input_type);
    println!("Output Type: {}", tool.output_type);
    println!("Enabled: {}", tool.enabled);
    if let Some(timeout) = tool.timeout_seconds {
        println!("Timeout: {timeout}s");
    }
    println!("Tags: {:?}", tool.tags);
    Ok(())
}

fn tool_explain(repo_dir: &Path, _db: &Connection, name: &str) -> Result<()> {
    let tool = Repository::load_tool_files(repo_dir)?
        .into_iter()
        .find(|t| t.name == name)
        .with_context(|| format!("tool '{name}' not found"))?;
    let command = if tool.command.is_empty() {
        format!("{} {}", tool.executable, tool.arguments.join(" "))
    } else {
        tool.command.clone()
    };
    let stages = parse_pipeline(&command)?;
    println!("Tool: {}\nSaved command: {}\nStages:", tool.name, command);
    for (index, stage) in stages.iter().enumerate() {
        println!("  {}. {}", index + 1, stage.join(" "));
    }
    let placeholders: Vec<&str> = ["{target}", "{asset}", "{input}", "{file}", "{scope}"]
        .into_iter()
        .filter(|p| command.contains(*p))
        .collect();
    println!(
        "Placeholders: {}",
        if placeholders.is_empty() {
            "none".to_owned()
        } else {
            placeholders.join(", ")
        }
    );
    println!("Execution is explicit only. Final stdout is recorded locally and optionally ingested as assets.");
    Ok(())
}

fn tool_add(repo_dir: &Path, db: &mut Connection, tool: ToolDefinition) -> Result<()> {
    tool.validate()?;
    Repository::save_tool_file(repo_dir, &tool)?;

    let seen = now()?;
    let args_json = serde_json::to_string(&tool.arguments)?;
    let tags_json = serde_json::to_string(&tool.tags)?;

    db.execute(
        "INSERT INTO tools (name, description, executable, arguments_json, input_type, output_type, enabled, timeout_seconds, tags_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)
         ON CONFLICT(name) DO UPDATE SET
           description=excluded.description, executable=excluded.executable,
           arguments_json=excluded.arguments_json, input_type=excluded.input_type,
           output_type=excluded.output_type, enabled=excluded.enabled,
           timeout_seconds=excluded.timeout_seconds, tags_json=excluded.tags_json,
           updated_at=excluded.updated_at",
        params![
            tool.name,
            tool.description,
            tool.executable,
            args_json,
            tool.input_type,
            tool.output_type,
            tool.enabled as i32,
            tool.timeout_seconds,
            tags_json,
            seen,
        ],
    )?;

    event(db, "tool.configured", "tool", &tool.name)?;
    println!("Configured tool: {}", tool.name);
    Ok(())
}

fn tool_remove(repo_dir: &Path, db: &mut Connection, name: &str) -> Result<()> {
    Repository::remove_tool_file(repo_dir, name)?;
    db.execute("DELETE FROM tools WHERE name=?1", [name])?;
    event(db, "tool.removed", "tool", name)?;
    println!("Removed tool: {name}");
    Ok(())
}

fn tool_validate(repo_dir: &Path, _db: &Connection, name: &str) -> Result<()> {
    let tools = Repository::load_tool_files(repo_dir)?;
    let tool = tools
        .into_iter()
        .find(|t| t.name == name)
        .with_context(|| format!("tool '{name}' not found"))?;

    println!("Validating tool: {name}");
    match tool.validate() {
        Ok(_) => println!("  ✓ Configuration syntax valid"),
        Err(e) => println!("  ✗ Configuration error: {e}"),
    }

    if check_executable_exists(&tool.executable) {
        println!("  ✓ Executable found: {}", tool.executable);
    } else {
        println!(
            "  ✗ Executable not found in PATH or disk: {}",
            tool.executable
        );
    }

    println!("  ✓ Input type: {}", tool.input_type);
    println!("  ✓ Output type: {}", tool.output_type);
    Ok(())
}

fn check_executable_exists(executable: &str) -> bool {
    let path = Path::new(executable);
    if path.is_file() {
        return true;
    }
    if let Ok(paths) = std::env::var("PATH") {
        for dir in std::env::split_paths(&paths) {
            let full_path = dir.join(executable);
            if full_path.is_file() {
                return true;
            }
            #[cfg(windows)]
            {
                let exe_path = dir.join(format!("{executable}.exe"));
                if exe_path.is_file() {
                    return true;
                }
                let cmd_path = dir.join(format!("{executable}.cmd"));
                if cmd_path.is_file() {
                    return true;
                }
                let bat_path = dir.join(format!("{executable}.bat"));
                if bat_path.is_file() {
                    return true;
                }
            }
        }
    }
    false
}

fn tool_run(
    repo_dir: &Path,
    db: &mut Connection,
    name: &str,
    target_override: Option<&str>,
    import_output: bool,
) -> Result<()> {
    let tools = Repository::load_tool_files(repo_dir)?;
    let tool = tools
        .into_iter()
        .find(|t| t.name == name)
        .with_context(|| format!("tool '{name}' not found"))?;

    if !tool.enabled {
        bail!("tool '{name}' is currently disabled");
    }

    if !tool.command.trim().is_empty() {
        return tool_run_pipeline(db, &tool, target_override, import_output);
    }

    let target_val = if let Some(t) = target_override {
        t.to_string()
    } else {
        let t_opt: Option<String> = db
            .query_row("SELECT value FROM targets LIMIT 1", [], |r| r.get(0))
            .optional()?;
        t_opt.with_context(|| "no target found. Add a target with `githunter target add <target>` or specify `--target <target>`")?
    };

    let scope_patterns: Vec<String> = {
        let mut stmt = db.prepare("SELECT pattern FROM scope_rules WHERE state='IN_SCOPE'")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        rows.collect::<rusqlite::Result<_>>()?
    };

    let resolved_args = tool.resolve_arguments(&target_val, &scope_patterns);

    println!(
        "⚡ [Opt-In Execution] Running '{}' with target '{}'...",
        tool.name, target_val
    );

    let mut cmd = std::process::Command::new(&tool.executable);
    cmd.args(&resolved_args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().with_context(|| {
        format!(
            "failed to start executable '{}'. Ensure it is installed and in PATH.",
            tool.executable
        )
    })?;

    let mut stdout_bytes = Vec::new();
    if let Some(mut out) = child.stdout.take() {
        out.read_to_end(&mut stdout_bytes)?;
    }

    let mut stderr_bytes = Vec::new();
    if let Some(mut err) = child.stderr.take() {
        err.read_to_end(&mut stderr_bytes)?;
    }

    let status = child.wait()?;
    if !status.success() {
        let err_msg = String::from_utf8_lossy(&stderr_bytes);
        eprintln!("Tool execution exited with status {status}. Stderr:\n{err_msg}");
    }

    event(db, "tool.executed", "tool", &tool.name)?;

    if import_output && !stdout_bytes.is_empty() {
        println!();
        println!("📥 Ingesting tool output into GitHunter asset pipeline...");
        import_assets_from_reader(db, std::io::Cursor::new(stdout_bytes), &tool.name)?;
    }

    Ok(())
}

fn tool_run_pipeline(
    db: &mut Connection,
    tool: &ToolDefinition,
    target_override: Option<&str>,
    import_output: bool,
) -> Result<()> {
    let target = if let Some(value) = target_override {
        value.to_owned()
    } else {
        db.query_row(
            "SELECT value FROM targets ORDER BY value LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .optional()?
        .context("no target found. Add a target or specify --target")?
    };
    let scope: Vec<String> = db
        .prepare("SELECT pattern FROM scope_rules WHERE state='IN_SCOPE' ORDER BY pattern")?
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let stages = parse_pipeline(&tool.command)?;
    println!(
        "[Opt-In Execution] Running pipeline '{}' with target '{}' ({} stages)...",
        tool.name,
        target,
        stages.len()
    );
    let started = now()?;
    let mut input = Vec::new();
    let mut stderr = Vec::new();
    let mut code = Some(0);
    for (index, stage) in stages.iter().enumerate() {
        if !check_executable_exists(&stage[0]) {
            bail!(
                "pipeline stage {} executable not found: {}",
                index + 1,
                stage[0]
            );
        }
        let args: Vec<String> = stage[1..]
            .iter()
            .map(|arg| {
                arg.replace("{target}", &target)
                    .replace("{asset}", &target)
                    .replace("{input}", &target)
                    .replace("{scope}", &scope.join(","))
            })
            .collect();
        let mut command = std::process::Command::new(&stage[0]);
        command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .with_context(|| format!("could not start pipeline stage {}", index + 1))?;
        if !input.is_empty() {
            use std::io::Write;
            child
                .stdin
                .take()
                .context("could not open stage stdin")?
                .write_all(&input)?;
        }
        let result = child.wait_with_output()?;
        code = result.status.code();
        stderr.extend_from_slice(&result.stderr);
        input = result.stdout;
        if !result.status.success() {
            break;
        }
    }
    let success = code == Some(0);
    db.execute(
        "INSERT INTO tool_executions VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![
            Uuid::new_v4().to_string(),
            tool.name,
            tool.command,
            target,
            started,
            now()?,
            if success { "success" } else { "failed" },
            code,
            String::from_utf8_lossy(&input),
            String::from_utf8_lossy(&stderr)
        ],
    )?;
    event(db, "tool.executed", "tool", &tool.name)?;
    if !success {
        bail!(
            "pipeline failed with exit {:?}: {}",
            code,
            String::from_utf8_lossy(&stderr)
        );
    }
    if import_output && !input.is_empty() {
        println!("Ingesting pipeline output into GitHunter asset pipeline...");
        import_assets_from_reader(db, io::Cursor::new(input), &format!("tool:{}", tool.name))?;
    }
    Ok(())
}

fn tool_run_all(
    repo_dir: &Path,
    db: &mut Connection,
    target_override: Option<&str>,
    import_output: bool,
) -> Result<()> {
    let tools = Repository::load_tool_files(repo_dir)?;
    let enabled_tools: Vec<_> = tools.into_iter().filter(|t| t.enabled).collect();
    if enabled_tools.is_empty() {
        println!("No enabled tools found to execute.");
        return Ok(());
    }

    println!("Running {} configured tools...", enabled_tools.len());
    for tool in enabled_tools {
        tool_run(repo_dir, db, &tool.name, target_override, import_output)?;
        println!("{}", "=".repeat(60));
    }
    Ok(())
}

fn workflow_list(repo_dir: &Path, _db: &Connection) -> Result<()> {
    let workflows = Repository::load_workflow_files(repo_dir)?;
    if workflows.is_empty() {
        println!("No workflows configured.");
        println!("Add workflows with `githunter workflow add --name <name> --steps <step1,step2>`");
        return Ok(());
    }

    println!("{:<20} {:<30} {:<30}", "WORKFLOW", "STEPS", "DESCRIPTION");
    println!("{}", "-".repeat(80));
    for wf in workflows {
        let steps_str = wf.steps.join(" -> ");
        println!("{:<20} {:<30} {:<30}", wf.name, steps_str, wf.description);
    }
    Ok(())
}

fn workflow_show(repo_dir: &Path, _db: &Connection, name: &str) -> Result<()> {
    let workflows = Repository::load_workflow_files(repo_dir)?;
    let wf = workflows
        .into_iter()
        .find(|w| w.name == name)
        .with_context(|| format!("workflow '{name}' not found"))?;

    println!("Workflow: {}", wf.name);
    println!("Description: {}", wf.description);
    println!("Steps: {:?}", wf.steps);
    Ok(())
}

fn workflow_add(repo_dir: &Path, db: &mut Connection, wf: WorkflowDefinition) -> Result<()> {
    wf.validate()?;
    Repository::save_workflow_file(repo_dir, &wf)?;

    let seen = now()?;
    let steps_json = serde_json::to_string(&wf.steps)?;

    db.execute(
        "INSERT INTO tool_workflows (name, description, steps_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)
         ON CONFLICT(name) DO UPDATE SET
           description=excluded.description, steps_json=excluded.steps_json, updated_at=excluded.updated_at",
        params![wf.name, wf.description, steps_json, seen],
    )?;

    event(db, "workflow.configured", "workflow", &wf.name)?;
    println!("Configured workflow: {}", wf.name);
    Ok(())
}

fn workflow_remove(repo_dir: &Path, db: &mut Connection, name: &str) -> Result<()> {
    Repository::remove_workflow_file(repo_dir, name)?;
    db.execute("DELETE FROM tool_workflows WHERE name=?1", [name])?;
    event(db, "workflow.removed", "workflow", name)?;
    println!("Removed workflow: {name}");
    Ok(())
}

fn workflow_run(
    repo_dir: &Path,
    db: &mut Connection,
    name: &str,
    target_override: Option<&str>,
) -> Result<()> {
    let workflows = Repository::load_workflow_files(repo_dir)?;
    let wf = workflows
        .into_iter()
        .find(|w| w.name == name)
        .with_context(|| format!("workflow '{name}' not found"))?;

    println!(
        "🚀 Executing workflow '{}' ({} steps)...",
        wf.name,
        wf.steps.len()
    );
    for (idx, step) in wf.steps.iter().enumerate() {
        println!("\n--- Step {}: Running tool '{}' ---", idx + 1, step);
        tool_run(repo_dir, db, step, target_override, true)?;
    }
    println!("\n✨ Workflow '{}' completed successfully!", wf.name);
    Ok(())
}

fn recommend(repo_dir: &Path, db: &Connection) -> Result<()> {
    let project: String = db
        .query_row("SELECT name FROM projects LIMIT 1", [], |r| r.get(0))
        .unwrap_or_else(|_| "Unknown".into());
    let target_count: i64 = db.query_row("SELECT COUNT(*) FROM targets", [], |r| r.get(0))?;
    let scope_count: i64 = db.query_row("SELECT COUNT(*) FROM scope_rules", [], |r| r.get(0))?;
    let asset_count: i64 = db.query_row("SELECT COUNT(*) FROM assets", [], |r| r.get(0))?;
    let snapshot_count: i64 = db.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))?;
    let tools = Repository::load_tool_files(repo_dir)?;

    println!("🎯 GITHUNTER RESEARCH RECOMMENDATIONS");
    println!("========================================");
    println!("Project: {project}");
    println!();
    println!("Current Research State:");
    println!("  • Targets:          {target_count}");
    println!("  • Scope Rules:      {scope_count}");
    println!("  • Tracked Assets:   {asset_count}");
    println!("  • Snapshots:        {snapshot_count}");
    println!("  • Configured Tools: {}", tools.len());
    println!();
    println!("Advisory Next Steps:");

    let mut step = 1;
    if target_count == 0 {
        println!("  {step}. Register primary authorized target: `githunter target add <domain>`");
        step += 1;
    }
    if scope_count == 0 {
        println!(
            "  {step}. Define In-Scope rules: `githunter scope add \"*.<domain>\"` or `githunter scope add --file scope.txt`"
        );
        step += 1;
    }
    if asset_count == 0 {
        println!(
            "  {step}. Ingest initial recon assets: `githunter asset import assets.txt --source <tool>` or cat assets.txt | `githunter asset import`"
        );
        step += 1;
    }
    if asset_count > 0 && snapshot_count == 0 {
        println!(
            "  {step}. Capture baseline research state: `githunter snapshot create --note \"Initial baseline\"`"
        );
        step += 1;
    }
    if tools.is_empty() {
        println!(
            "  {step}. Configure passive discovery tools: `githunter tool add \"subfinder -d {{target}} -silent\" --name subfinder`"
        );
        step += 1;
    } else if asset_count > 0 && snapshot_count > 0 {
        println!(
            "  {step}. Run configured recon tools: `githunter tool run <name>` or `githunter tool run all`"
        );
        step += 1;
        println!("  {step}. Compare changes against baseline: `githunter diff`");
    }
    let _ = step;
    println!();
    println!("* Note: GitHunter is local-first. Tool executions and recommendations are strictly advisory and require explicit opt-in.");
    Ok(())
}

fn snapshot_create(db: &mut Connection, note: Option<&str>) -> Result<()> {
    let assets: Vec<(String, String, String, String)> = {
        let mut statement = db.prepare(
            "SELECT id,asset_type,normalized_value,scope_status FROM assets ORDER BY asset_type,normalized_value",
        )?;
        let assets = statement
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        assets
    };
    let manifest = serde_json::to_vec(&assets)?;
    let hash = format!("{:x}", Sha256::digest(&manifest));
    let sequence: i64 = db.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))?;
    let id = Uuid::new_v4().to_string();
    let display = format!("s_{:04}", sequence + 1);
    let tx = db.transaction()?;
    tx.execute(
        "INSERT INTO snapshots VALUES (?1,?2,?3,?4,?5,?6)",
        params![id, display, hash, now()?, assets.len() as i64, note],
    )?;
    for (asset_id, kind, value, state) in assets {
        let asset_hash = format!("{:x}", Sha256::digest(format!("{kind}:{value}:{state}")));
        tx.execute(
            "INSERT INTO snapshot_assets VALUES (?1,?2,?3)",
            params![id, asset_id, asset_hash],
        )?;
    }
    tx.commit()?;
    event(db, "snapshot.created", "snapshot", &id)?;
    println!("Created snapshot: {display}");
    Ok(())
}

fn snapshot_list(db: &Connection) -> Result<()> {
    let mut s = db.prepare(
        "SELECT display_id,created_at,asset_count,COALESCE(note,'') FROM snapshots ORDER BY created_at",
    )?;
    for row in s.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, String>(3)?,
        ))
    })? {
        let (a, b, c, d) = row?;
        println!("{a}  {b}  assets: {c}  {d}");
    }
    Ok(())
}

/// Creates a new immutable snapshot from the union of two existing snapshots.
/// If an asset has a distinct historic hash in both parents, the second argument
/// is authoritative, mirroring a conventional `base -> updated` merge flow.
fn snapshot_merge(db: &mut Connection, first: &str, second: &str) -> Result<()> {
    if first == second {
        bail!("snapshot merge requires two different snapshots");
    }

    let find_snapshot = |display_id: &str| -> Result<(String, String)> {
        db.query_row(
            "SELECT id,display_id FROM snapshots WHERE display_id=?1",
            [display_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .with_context(|| {
            format!("snapshot '{display_id}' not found; use `githunter snapshot list`")
        })
    };
    let (first_id, first_display) = find_snapshot(first)?;
    let (second_id, second_display) = find_snapshot(second)?;

    let mut merged_assets: HashMap<String, String> = HashMap::new();
    for parent_id in [&first_id, &second_id] {
        let mut statement =
            db.prepare("SELECT asset_id,asset_hash FROM snapshot_assets WHERE snapshot_id=?1")?;
        for row in statement.query_map([parent_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })? {
            let (asset_id, asset_hash) = row?;
            // Iterating the second parent last intentionally resolves state ties.
            merged_assets.insert(asset_id, asset_hash);
        }
    }
    let mut assets: Vec<(String, String)> = merged_assets.into_iter().collect();
    assets.sort_by(|left, right| left.0.cmp(&right.0));

    let mut parents = vec![first_id, second_id];
    parents.sort();
    let manifest = serde_json::to_vec(&json!({
        "kind": "snapshot_merge",
        "parents": parents,
        "assets": assets,
    }))?;
    let manifest_hash = format!("{:x}", Sha256::digest(&manifest));
    let sequence: i64 = db.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))?;
    let id = Uuid::new_v4().to_string();
    let display = format!("s_{:04}", sequence + 1);
    let note = format!("merge of {first_display} and {second_display}");
    let timestamp = now()?;
    let transaction = db.transaction()?;
    transaction.execute(
        "INSERT INTO snapshots VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            id,
            display,
            manifest_hash,
            timestamp,
            assets.len() as i64,
            note
        ],
    )?;
    for (asset_id, asset_hash) in assets {
        transaction.execute(
            "INSERT INTO snapshot_assets VALUES (?1,?2,?3)",
            params![id, asset_id, asset_hash],
        )?;
    }
    transaction.commit()?;
    event(db, "snapshot.merged", "snapshot", &id)?;
    println!("Merged snapshots {first_display} + {second_display}: {display}");
    Ok(())
}

fn diff(db: &Connection) -> Result<()> {
    let mut s =
        db.prepare("SELECT id,display_id FROM snapshots ORDER BY created_at DESC LIMIT 2")?;
    let snaps: Vec<(String, String)> = s
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    if snaps.len() < 2 {
        bail!("two snapshots are required for a diff");
    }
    let count = |id: &str| -> Result<i64> {
        Ok(db.query_row(
            "SELECT COUNT(*) FROM snapshot_assets WHERE snapshot_id=?1 AND asset_id NOT IN (SELECT asset_id FROM snapshot_assets WHERE snapshot_id=?2)",
            params![id, snaps[1].0],
            |r| r.get(0),
        )?)
    };
    let added = count(&snaps[0].0)?;
    let removed: i64 = db.query_row(
        "SELECT COUNT(*) FROM snapshot_assets WHERE snapshot_id=?1 AND asset_id NOT IN (SELECT asset_id FROM snapshot_assets WHERE snapshot_id=?2)",
        params![snaps[1].0, snaps[0].0],
        |row| row.get(0),
    )?;
    println!(
        "Diff {} → {}\nAdded: {added}\nRemoved: {removed}",
        snaps[1].1, snaps[0].1
    );
    Ok(())
}

fn status(db: &Connection) -> Result<()> {
    let project: String = db.query_row("SELECT name FROM projects LIMIT 1", [], |r| r.get(0))?;
    let assets: i64 = db.query_row("SELECT COUNT(*) FROM assets", [], |r| r.get(0))?;
    let snapshots: i64 = db.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))?;
    println!("GITHUNTER STATUS\n\nProject: {project}\nAssets: {assets}\nSnapshots: {snapshots}");
    Ok(())
}

/// Renders local state only. It does not start tools, scans, or network activity.
fn watch(repo_dir: &Path, db: &Connection, interval_seconds: u64, once: bool) -> Result<()> {
    if interval_seconds == 0 {
        bail!("--interval must be at least 1 second");
    }
    loop {
        // A number of terminal hosts expose stdout as a redirected stream even
        // though they still interpret ANSI control sequences.  Do not gate the
        // refresh on `is_terminal()` or those hosts append a full dashboard on
        // every interval. `--once` remains plain text for scripts.
        if !once {
            print!("\x1b[2J\x1b[H");
        }
        watch_frame(repo_dir, db, interval_seconds)?;
        io::stdout().flush()?;
        if once {
            return Ok(());
        }
        thread::sleep(Duration::from_secs(interval_seconds));
    }
}

fn watch_frame(repo_dir: &Path, db: &Connection, interval_seconds: u64) -> Result<()> {
    let project: String = db.query_row("SELECT name FROM projects LIMIT 1", [], |r| r.get(0))?;
    let targets: i64 = db.query_row("SELECT COUNT(*) FROM targets", [], |r| r.get(0))?;
    let snapshots: i64 = db.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))?;
    let tools = Repository::load_tool_files(repo_dir)?;
    let asset_counts = |scope: &str| -> Result<i64> {
        Ok(db.query_row(
            "SELECT COUNT(*) FROM assets WHERE scope_status=?1",
            [scope],
            |r| r.get(0),
        )?)
    };
    let scope_counts = |state: &str| -> Result<i64> {
        Ok(db.query_row(
            "SELECT COUNT(*) FROM scope_rules WHERE state=?1",
            [state],
            |r| r.get(0),
        )?)
    };
    let total_assets: i64 = db.query_row("SELECT COUNT(*) FROM assets", [], |r| r.get(0))?;

    println!("GITHUNTER LIVE DASHBOARD");
    println!("Project: {project}    Updated: {}", now()?);
    println!(
        "Refresh: every {interval_seconds} seconds (Ctrl+C to stop; use --interval to change)"
    );
    println!("{}", "=".repeat(72));
    println!("OVERVIEW");
    println!(
        "  Targets: {targets:<5}  Scope rules: in {} / out {}",
        scope_counts("IN_SCOPE")?,
        scope_counts("OUT_OF_SCOPE")?
    );
    println!(
        "  Assets:  {total_assets:<5}  Snapshots: {snapshots:<5}  Configured tools: {}",
        tools.len()
    );
    println!(
        "  Asset scope: in {} / out {} / unknown {}",
        asset_counts("IN_SCOPE")?,
        asset_counts("OUT_OF_SCOPE")?,
        asset_counts("UNKNOWN")?
    );

    println!();
    println!("ASSET TYPES");
    let mut type_statement = db.prepare(
        "SELECT asset_type, COUNT(*) FROM assets GROUP BY asset_type ORDER BY asset_type",
    )?;
    let types: Vec<(String, i64)> = type_statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    if types.is_empty() {
        println!("  No tracked assets yet.");
    } else {
        for (kind, count) in types {
            println!("  {kind:<12} {count}");
        }
    }

    println!();
    println!("RECENT ACTIVITY");
    let mut events_statement = db.prepare(
        "SELECT occurred_at,event_type,entity_type FROM timeline_events ORDER BY occurred_at DESC LIMIT 8",
    )?;
    let events: Vec<(String, String, String)> = events_statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    if events.is_empty() {
        println!("  No activity recorded yet.");
    } else {
        for (occurred_at, event_type, entity_type) in events {
            println!("  {occurred_at}  {event_type} ({entity_type})");
        }
    }
    println!();
    println!("Read-only monitor: external tools run only through explicit `githunter tool run` commands.");
    Ok(())
}

fn timeline(db: &Connection) -> Result<()> {
    let mut s = db.prepare(
        "SELECT occurred_at,event_type,entity_type FROM timeline_events ORDER BY occurred_at",
    )?;
    for row in s.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })? {
        let (a, b, c) = row?;
        println!("{a}  {b} ({c})");
    }
    Ok(())
}
