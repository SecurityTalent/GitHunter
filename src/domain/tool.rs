use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

fn default_input_type() -> String {
    "target".to_string()
}

fn default_output_type() -> String {
    "lines".to_string()
}

fn default_true() -> bool {
    true
}

/// A configurable external security tool definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDefinition {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub executable: String,
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default = "default_input_type")]
    pub input_type: String,
    #[serde(default = "default_output_type")]
    pub output_type: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl ToolDefinition {
    /// Validates the tool configuration for safety and completeness.
    pub fn validate(&self) -> Result<()> {
        let name = self.name.trim();
        if name.is_empty() {
            bail!("tool name cannot be empty");
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            bail!("tool name contains invalid characters: only alphanumeric, hyphen, and underscore allowed");
        }
        if self.executable.trim().is_empty() {
            bail!("tool executable cannot be empty");
        }
        // Validate input and output types
        let input = self.input_type.trim().to_ascii_lowercase();
        if !["target", "scope", "file", "stdin", "none"].contains(&input.as_str()) {
            bail!("unsupported input_type '{input}': expected 'target', 'scope', 'file', 'stdin', or 'none'");
        }
        let output = self.output_type.trim().to_ascii_lowercase();
        if !["lines", "json"].contains(&output.as_str()) {
            bail!("unsupported output_type '{output}': expected 'lines' or 'json'");
        }
        Ok(())
    }

    /// Resolves argument template variables ({target}, {scope}) safely.
    pub fn resolve_arguments(&self, target: &str, scope: &[String]) -> Vec<String> {
        let scope_joined = scope.join(",");
        self.arguments
            .iter()
            .map(|arg| {
                arg.replace("{target}", target)
                    .replace("{scope}", &scope_joined)
            })
            .collect()
    }
}

/// An ordered workflow of registered tools.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowDefinition {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub steps: Vec<String>,
}

impl WorkflowDefinition {
    pub fn validate(&self) -> Result<()> {
        let name = self.name.trim();
        if name.is_empty() {
            bail!("workflow name cannot be empty");
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            bail!("workflow name contains invalid characters");
        }
        if self.steps.is_empty() {
            bail!("workflow must contain at least one tool step");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_tool_configuration() {
        let tool = ToolDefinition {
            name: "subfinder".into(),
            description: "Subdomain discovery".into(),
            executable: "subfinder".into(),
            arguments: vec!["-d".into(), "{target}".into(), "-silent".into()],
            input_type: "target".into(),
            output_type: "lines".into(),
            enabled: true,
            timeout_seconds: Some(60),
            tags: vec!["subdomain-discovery".into()],
        };
        assert!(tool.validate().is_ok());

        let resolved = tool.resolve_arguments("example.com", &[]);
        assert_eq!(resolved, vec!["-d", "example.com", "-silent"]);
    }

    #[test]
    fn rejects_invalid_tool_names() {
        let invalid = ToolDefinition {
            name: "tool with spaces; rm -rf".into(),
            description: "".into(),
            executable: "subfinder".into(),
            arguments: vec![],
            input_type: "target".into(),
            output_type: "lines".into(),
            enabled: true,
            timeout_seconds: None,
            tags: vec![],
        };
        assert!(invalid.validate().is_err());
    }
}
