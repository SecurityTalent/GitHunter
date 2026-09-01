use anyhow::{bail, Context, Result};
use clap::Args;
use std::path::PathBuf;

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

    let name = args.name.unwrap_or_else(|| {
        root.file_name()
            .and_then(|part| part.to_str())
            .unwrap_or("githunter-project")
            .to_owned()
    });
    let repository = Repository::initialize(&root, &name)?;

    println!("Initialized GitHunter repository.");
    println!();
    println!("Project: {}", repository.project_name());
    println!("Location: {}", repository.githunter_dir().display());
    println!();
    println!("Repository setup is complete.");
    println!("Run `githunter --help` to see the commands available in this version.");
    Ok(())
}
