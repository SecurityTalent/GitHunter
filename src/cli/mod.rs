mod init;

use crate::application::research_state;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "githunter",
    version,
    about = "Offline Security Research Version Control",
    long_about = "GitHunter records authorized security research locally. It never executes reconnaissance or external security tools."
)]
pub struct Cli {
    /// Use an explicit project directory instead of the current directory.
    #[arg(long, global = true, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Disable ANSI color output.
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize a GitHunter repository in the selected directory.
    Init(init::InitArgs),
    /// Display project metadata.
    Project(research_state::ProjectArgs),
    /// Manage authorized targets.
    Target(research_state::TargetArgs),
    /// Manage explicit in-scope and out-of-scope rules.
    Scope(research_state::ScopeArgs),
    /// Ingest, manage, and inspect observed assets.
    Asset(research_state::AssetArgs),
    /// Manage and inspect external security tools.
    Tool(research_state::ToolArgs),
    /// Manage and run automated tool workflows.
    Workflow(research_state::WorkflowArgs),
    /// Advisory recommendations based on project state.
    Recommend,
    /// Generate shell completion scripts.
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Create and inspect immutable security-state snapshots.
    Snapshot(research_state::SnapshotArgs),
    /// Compare the two most recent security-state snapshots.
    Diff,
    /// Display the current research-state summary.
    Status,
    /// Display immutable project history.
    Timeline,
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init(args) => init::execute(cli.repo, args),
        Commands::Project(args) => {
            research_state::execute(cli.repo, research_state::P0Command::Project(args))
        }
        Commands::Target(args) => {
            research_state::execute(cli.repo, research_state::P0Command::Target(args))
        }
        Commands::Scope(args) => {
            research_state::execute(cli.repo, research_state::P0Command::Scope(args))
        }
        Commands::Asset(args) => {
            research_state::execute(cli.repo, research_state::P0Command::Asset(args))
        }
        Commands::Tool(args) => {
            research_state::execute(cli.repo, research_state::P0Command::Tool(args))
        }
        Commands::Workflow(args) => {
            research_state::execute(cli.repo, research_state::P0Command::Workflow(args))
        }
        Commands::Recommend => {
            research_state::execute(cli.repo, research_state::P0Command::Recommend)
        }
        Commands::Completions { shell } => {
            research_state::execute(cli.repo, research_state::P0Command::Completions { shell })
        }
        Commands::Snapshot(args) => {
            research_state::execute(cli.repo, research_state::P0Command::Snapshot(args))
        }
        Commands::Diff => research_state::execute(cli.repo, research_state::P0Command::Diff),
        Commands::Status => research_state::execute(cli.repo, research_state::P0Command::Status),
        Commands::Timeline => {
            research_state::execute(cli.repo, research_state::P0Command::Timeline)
        }
    }
}
