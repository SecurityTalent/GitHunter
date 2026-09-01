use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::domain::asset::{asset_type, matches_pattern, normalize, normalize_pattern};
use crate::repository::Repository;

#[derive(Debug, Subcommand)]
pub enum P0Command {
    /// Display project metadata.
    Project(ProjectArgs),
    /// Manage authorized targets.
    Target(TargetArgs),
    /// Manage explicit in-scope and out-of-scope rules.
    Scope(ScopeArgs),
    /// Import and inspect observed assets.
    Asset(AssetArgs),
    /// Create and inspect immutable security-state snapshots.
    Snapshot(SnapshotArgs),
    /// Compare the most recent two snapshots.
    Diff,
    /// Display the current research-state summary.
    Status,
    /// Display immutable project history.
    Timeline,
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
enum ScopeCommand {
    Add {
        pattern: String,
    },
    Out {
        #[command(subcommand)]
        command: ScopeOutCommand,
    },
    List,
    Check {
        value: String,
    },
}
#[derive(Debug, Subcommand)]
enum ScopeOutCommand {
    Add { pattern: String },
}
#[derive(Debug, Args)]
pub struct AssetArgs {
    #[command(subcommand)]
    command: AssetCommand,
}
#[derive(Debug, Subcommand)]
enum AssetCommand {
    Import {
        file: PathBuf,
        #[arg(long, default_value = "file")]
        source: String,
    },
    List,
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
}

pub fn execute(path: Option<PathBuf>, command: P0Command) -> Result<()> {
    let root = path
        .unwrap_or(std::env::current_dir().context("could not determine the current directory")?);
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
            ScopeCommand::Add { pattern } => scope_add(&mut db, &pattern, "IN_SCOPE"),
            ScopeCommand::Out {
                command: ScopeOutCommand::Add { pattern },
            } => scope_add(&mut db, &pattern, "OUT_OF_SCOPE"),
            ScopeCommand::List => scope_list(&db),
            ScopeCommand::Check { value } => {
                println!("{}  {}", scope_status(&db, &value)?, value);
                Ok(())
            }
        },
        P0Command::Asset(args) => match args.command {
            AssetCommand::Import { file, source } => asset_import(&mut db, &file, &source),
            AssetCommand::List => asset_list(&db),
        },
        P0Command::Snapshot(args) => match args.command {
            SnapshotCommand::Create { note } => snapshot_create(&mut db, note.as_deref()),
            SnapshotCommand::List => snapshot_list(&db),
        },
        P0Command::Diff => diff(&db),
        P0Command::Status => status(&db),
        P0Command::Timeline => timeline(&db),
    }
}

fn now() -> Result<String> {
    Ok(OffsetDateTime::now_utc().format(&Rfc3339)?)
}
fn event(db: &rusqlite::Connection, kind: &str, entity: &str, id: &str) -> Result<()> {
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
fn project_show(db: &rusqlite::Connection) -> Result<()> {
    let (name, id, created): (String, String, String) =
        db.query_row("SELECT name,id,created_at FROM projects LIMIT 1", [], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
    println!("Project: {name}\nID: {id}\nCreated: {created}\nSchema: 2");
    Ok(())
}
fn target_add(db: &mut rusqlite::Connection, value: &str, authorization: &str) -> Result<()> {
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
fn target_list(db: &rusqlite::Connection) -> Result<()> {
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
fn scope_add(db: &mut rusqlite::Connection, pattern: &str, state: &str) -> Result<()> {
    let p = normalize_pattern(pattern)?;
    let id = Uuid::new_v4().to_string();
    db.execute(
        "INSERT INTO scope_rules VALUES (?1,?2,?3,?4,?5)",
        params![id, p, state, "project scope", now()?],
    )
    .context("scope rule already exists or is invalid")?;
    event(db, "scope.created", "scope_rule", &id)?;
    println!("Added {state}: {p}");
    Ok(())
}
fn scope_list(db: &rusqlite::Connection) -> Result<()> {
    let mut s = db.prepare("SELECT state,pattern FROM scope_rules ORDER BY state,pattern")?;
    for row in s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))? {
        let (a, b) = row?;
        println!("{a:<12} {b}");
    }
    Ok(())
}
fn scope_status(db: &rusqlite::Connection, value: &str) -> Result<String> {
    let v = normalize(value)?;
    let mut s = db.prepare("SELECT pattern,state FROM scope_rules")?;
    let rules = s.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut inside = false;
    for rule in rules {
        let (p, state) = rule?;
        if matches_pattern(&p, &v) {
            if state == "OUT_OF_SCOPE" {
                return Ok(state);
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
fn asset_import(db: &mut rusqlite::Connection, file: &PathBuf, source: &str) -> Result<()> {
    let content =
        fs::read_to_string(file).with_context(|| format!("could not read {}", file.display()))?;
    let tx = db.transaction()?;
    let mut added = 0;
    let mut existing = 0;
    let mut invalid = 0;
    for raw in content.lines().map(str::trim).filter(|v| !v.is_empty()) {
        let value = match normalize(raw) {
            Ok(v) => v,
            Err(_) => {
                invalid += 1;
                continue;
            }
        };
        let kind = asset_type(&value);
        let seen = now()?;
        let status = scope_status(&tx, &value)?;
        let id: Option<String> = tx
            .query_row(
                "SELECT id FROM assets WHERE asset_type=?1 AND normalized_value=?2",
                params![kind, value],
                |r| r.get(0),
            )
            .optional()?;
        let asset_id = match id {
            Some(id) => {
                existing += 1;
                tx.execute(
                    "UPDATE assets SET last_seen=?1 WHERE id=?2",
                    params![seen, id],
                )?;
                id
            }
            None => {
                added += 1;
                let id = Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO assets VALUES (?1,?2,?3,?4,?5,?5,'{}')",
                    params![id, kind, value, status, seen],
                )?;
                id
            }
        };
        tx.execute(
            "INSERT INTO asset_observations VALUES (?1,?2,?3,?4,?5,'{}')",
            params![Uuid::new_v4().to_string(), asset_id, raw, source, seen],
        )?;
    }
    tx.commit()?;
    event(db, "assets.imported", "asset_import", source)?;
    println!("Imported assets. New: {added}, existing: {existing}, invalid: {invalid}");
    Ok(())
}
fn asset_list(db: &rusqlite::Connection) -> Result<()> {
    let mut s=db.prepare("SELECT scope_status,asset_type,normalized_value FROM assets ORDER BY asset_type,normalized_value")?;
    for row in s.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })? {
        let (a, b, c) = row?;
        println!("{a:<12} {b:<6} {c}");
    }
    Ok(())
}
fn snapshot_create(db: &mut rusqlite::Connection, note: Option<&str>) -> Result<()> {
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
fn snapshot_list(db: &rusqlite::Connection) -> Result<()> {
    let mut s=db.prepare("SELECT display_id,created_at,asset_count,COALESCE(note,'') FROM snapshots ORDER BY created_at")?;
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
fn diff(db: &rusqlite::Connection) -> Result<()> {
    let mut s =
        db.prepare("SELECT id,display_id FROM snapshots ORDER BY created_at DESC LIMIT 2")?;
    let snaps: Vec<(String, String)> = s
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    if snaps.len() < 2 {
        bail!("two snapshots are required for a diff");
    }
    let count = |id: &str| -> Result<i64> {
        Ok(db.query_row("SELECT COUNT(*) FROM snapshot_assets WHERE snapshot_id=?1 AND asset_id NOT IN (SELECT asset_id FROM snapshot_assets WHERE snapshot_id=?2)",params![id,snaps[1].0],|r|r.get(0))?)
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
fn status(db: &rusqlite::Connection) -> Result<()> {
    let project: String = db.query_row("SELECT name FROM projects LIMIT 1", [], |r| r.get(0))?;
    let assets: i64 = db.query_row("SELECT COUNT(*) FROM assets", [], |r| r.get(0))?;
    let snapshots: i64 = db.query_row("SELECT COUNT(*) FROM snapshots", [], |r| r.get(0))?;
    println!("GITHUNTER STATUS\n\nProject: {project}\nAssets: {assets}\nSnapshots: {snapshots}");
    Ok(())
}
fn timeline(db: &rusqlite::Connection) -> Result<()> {
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
