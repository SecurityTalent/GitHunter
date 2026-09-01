use anyhow::{bail, Context, Result};
use clap::Args;
use rusqlite::params;
use std::io::{self, Write};
use std::path::PathBuf;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::domain::asset::{asset_type, normalize, normalize_pattern};
use crate::repository::Repository;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// A human-readable project name. Defaults to the directory name.
    #[arg(long)]
    pub name: Option<String>,
}

pub fn execute(repo_path: Option<PathBuf>, args: InitArgs) -> Result<()> {
    let root = match repo_path {
        Some(path) => path,
        None => std::env::current_dir().context("could not determine the current directory")?,
    };

    if !root.is_dir() {
        bail!("project directory does not exist: {}", root.display());
    }

    let default_name = root
        .file_name()
        .and_then(|part| part.to_str())
        .unwrap_or("githunter-project")
        .to_owned();

    // `--name` is deliberately non-interactive so scripts and CI remain stable.
    let setup = match args.name {
        Some(name) => InitialSetup {
            name,
            ..InitialSetup::default()
        },
        None => prompt_for_setup(&default_name)?,
    };
    let repository = Repository::initialize(&root, &setup.name)?;
    save_initial_setup(&root, &setup)?;

    println!("Initialized GitHunter repository.");
    println!();
    println!("Project: {}", repository.project_name());
    println!("Location: {}", repository.githunter_dir().display());
    println!();
    println!("Repository setup is complete.");
    println!("Run `githunter --help` to see the commands available in this version.");
    Ok(())
}

#[derive(Default)]
struct InitialSetup {
    name: String,
    primary_target: String,
    authorization_note: String,
    in_scope: String,
    out_of_scope: String,
}

fn prompt(label: &str, default: Option<&str>) -> Result<String> {
    match default {
        Some(value) => print!("{label} [{value}]: "),
        None => print!("{label}: "),
    }
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let answer = answer.trim().to_owned();
    Ok(if answer.is_empty() {
        default.unwrap_or_default().to_owned()
    } else {
        answer
    })
}

fn prompt_for_setup(default_name: &str) -> Result<InitialSetup> {
    println!("GitHunter project setup");
    println!("Enter comma-separated domains for scope fields, or leave them blank.");
    Ok(InitialSetup {
        name: prompt("Project name", Some(default_name))?,
        primary_target: prompt("Primary target", None)?,
        authorization_note: prompt("Authorization note", None)?,
        in_scope: prompt("In-scope domains", None)?,
        out_of_scope: prompt("Out-of-scope domains", None)?,
    })
}

fn scope_values(raw: &str) -> impl Iterator<Item = &str> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn save_initial_setup(root: &std::path::Path, setup: &InitialSetup) -> Result<()> {
    if setup.primary_target.trim().is_empty()
        && setup.in_scope.trim().is_empty()
        && setup.out_of_scope.trim().is_empty()
    {
        return Ok(());
    }

    let mut db = Repository::open(root)?;
    let transaction = db.transaction()?;
    let timestamp = OffsetDateTime::now_utc().format(&Rfc3339)?;

    if !setup.primary_target.trim().is_empty() {
        let target = normalize(&setup.primary_target)
            .with_context(|| format!("invalid primary target '{}'", setup.primary_target))?;
        let authorization = if setup.authorization_note.trim().is_empty() {
            "Authorized project target"
        } else {
            setup.authorization_note.trim()
        };
        transaction.execute(
            "INSERT INTO targets VALUES (?1,?2,?3,?4,?5)",
            params![
                Uuid::new_v4().to_string(),
                target,
                asset_type(&target),
                authorization,
                timestamp
            ],
        )?;
    }

    for (raw, state) in [
        (&setup.in_scope, "IN_SCOPE"),
        (&setup.out_of_scope, "OUT_OF_SCOPE"),
    ] {
        for value in scope_values(raw) {
            let pattern = normalize_pattern(value)
                .with_context(|| format!("invalid {state} domain '{value}'"))?;
            transaction.execute(
                "INSERT INTO scope_rules VALUES (?1,?2,?3,?4,?5)",
                params![
                    Uuid::new_v4().to_string(),
                    pattern,
                    state,
                    "init prompt",
                    timestamp
                ],
            )?;
        }
    }
    transaction.commit()?;
    Ok(())
}
