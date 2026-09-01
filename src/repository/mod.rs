use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::database;
use crate::domain::tool::{ToolDefinition, WorkflowDefinition};

const GITHUNTER_DIRECTORY: &str = ".githunter";

#[derive(Debug)]
pub struct Repository {
    githunter_dir: PathBuf,
    project_name: String,
}

#[derive(Serialize)]
struct Config<'a> {
    schema_version: i64,
    project_name: &'a str,
}

#[derive(Serialize)]
struct ProjectMetadata<'a> {
    id: &'a str,
    name: &'a str,
    schema_version: i64,
    created_at: &'a str,
}

impl Repository {
    pub fn initialize(root: &Path, project_name: &str) -> Result<Self> {
        let githunter_dir = root.join(GITHUNTER_DIRECTORY);
        if githunter_dir.exists() {
            bail!(
                "GitHunter repository already exists at {}",
                githunter_dir.display()
            );
        }
        if project_name.trim().is_empty() {
            bail!("project name cannot be empty");
        }

        fs::create_dir(&githunter_dir)
            .with_context(|| format!("could not create {}", githunter_dir.display()))?;
        // If initialization is interrupted, leave the visible marker directory in place rather
        // than deleting potentially useful diagnostic state.
        Self::initialize_contents(&githunter_dir, project_name)?;
        Ok(Self {
            githunter_dir,
            project_name: project_name.to_owned(),
        })
    }

    fn initialize_contents(githunter_dir: &Path, project_name: &str) -> Result<()> {
        for directory in [
            "objects/sha256",
            "locks",
            "backups",
            "metadata",
            "tools",
            "workflows",
        ] {
            fs::create_dir_all(githunter_dir.join(directory))?;
        }
        let project_id = Uuid::new_v4().to_string();
        let now = unix_timestamp()?;
        let config = toml::to_string_pretty(&Config {
            schema_version: database::SCHEMA_VERSION,
            project_name,
        })?;
        fs::write(githunter_dir.join("config.toml"), config)?;
        database::initialize(
            &githunter_dir.join("githunter.db"),
            &project_id,
            project_name,
            &now,
        )?;
        let metadata = serde_json::to_vec_pretty(&ProjectMetadata {
            id: &project_id,
            name: project_name,
            schema_version: database::SCHEMA_VERSION,
            created_at: &now,
        })?;
        fs::write(
            githunter_dir.join("metadata").join("project.json"),
            metadata,
        )?;
        Ok(())
    }

    pub fn githunter_dir(&self) -> &Path {
        &self.githunter_dir
    }

    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    pub fn discover(start: &Path) -> Result<PathBuf> {
        let start = start
            .canonicalize()
            .with_context(|| format!("could not access {}", start.display()))?;
        for directory in start.ancestors() {
            let candidate = directory.join(GITHUNTER_DIRECTORY);
            if candidate.join("githunter.db").is_file() {
                return Ok(candidate);
            }
        }
        bail!("GitHunter repository not found. Run `githunter init` first.")
    }

    pub fn open(start: &Path) -> Result<Connection> {
        let githunter_dir = Self::discover(start)?;
        let mut connection = Connection::open(githunter_dir.join("githunter.db"))?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        let now = unix_timestamp()?;
        database::migrate(&mut connection, &now)?;
        Ok(connection)
    }

    pub fn tools_dir(gitgithunter_dir: &Path) -> PathBuf {
        let dir = gitgithunter_dir.join("tools");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    pub fn workflows_dir(gitgithunter_dir: &Path) -> PathBuf {
        let dir = gitgithunter_dir.join("workflows");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    pub fn save_tool_file(gitgithunter_dir: &Path, tool: &ToolDefinition) -> Result<()> {
        let dir = Self::tools_dir(gitgithunter_dir);
        let path = dir.join(format!("{}.toml", tool.name));
        let content = toml::to_string_pretty(tool)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_tool_files(gitgithunter_dir: &Path) -> Result<Vec<ToolDefinition>> {
        let dir = Self::tools_dir(gitgithunter_dir);
        let mut tools = Vec::new();
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    let content = fs::read_to_string(&path)?;
                    if let Ok(tool) = toml::from_str::<ToolDefinition>(&content) {
                        tools.push(tool);
                    }
                }
            }
        }
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tools)
    }

    pub fn remove_tool_file(gitgithunter_dir: &Path, name: &str) -> Result<()> {
        let dir = Self::tools_dir(gitgithunter_dir);
        let path = dir.join(format!("{name}.toml"));
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn save_workflow_file(
        gitgithunter_dir: &Path,
        workflow: &WorkflowDefinition,
    ) -> Result<()> {
        let dir = Self::workflows_dir(gitgithunter_dir);
        let path = dir.join(format!("{}.toml", workflow.name));
        let content = toml::to_string_pretty(workflow)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_workflow_files(gitgithunter_dir: &Path) -> Result<Vec<WorkflowDefinition>> {
        let dir = Self::workflows_dir(gitgithunter_dir);
        let mut workflows = Vec::new();
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    let content = fs::read_to_string(&path)?;
                    if let Ok(wf) = toml::from_str::<WorkflowDefinition>(&content) {
                        workflows.push(wf);
                    }
                }
            }
        }
        workflows.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(workflows)
    }

    pub fn remove_workflow_file(gitgithunter_dir: &Path, name: &str) -> Result<()> {
        let dir = Self::workflows_dir(gitgithunter_dir);
        let path = dir.join(format!("{name}.toml"));
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

pub fn unix_timestamp() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("could not format the current timestamp")
}
