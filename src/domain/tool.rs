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
    /// Original one-line command or pipeline. Legacy definitions leave this empty.
    #[serde(default)]
    pub command: String,
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
        if self.executable.trim().is_empty() && self.command.trim().is_empty() {
            bail!("tool executable cannot be empty");
        }
        if !self.command.trim().is_empty() {
            parse_pipeline(&self.command)?;
        }
        if self.timeout_seconds == Some(0) {
            bail!("tool timeout must be at least 1 second");
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

/// A shell-free parser for a familiar command line. It supports quotes and `|`, but
/// deliberately rejects shell control/redirection syntax: saved commands are never
/// evaluated by a shell.
pub fn parse_pipeline(command: &str) -> Result<Vec<Vec<String>>> {
    let mut stages = vec![Vec::new()];
    let mut word = String::new();
    let mut quote = None;
    let mut escaped = false;
    for ch in command.chars() {
        if escaped {
            word.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            } else {
                word.push(ch);
            }
            continue;
        }
        if ch == '\'' || ch == '"' {
            quote = Some(ch);
            continue;
        }
        if matches!(ch, ';' | '&' | '`' | '>' | '<' | '\n' | '\r') {
            bail!("saved commands do not allow shell operators or redirection");
        }
        if ch == '|' {
            if !word.is_empty() {
                stages
                    .last_mut()
                    .expect("stage")
                    .push(std::mem::take(&mut word));
            }
            if stages.last().map_or(true, Vec::is_empty) {
                bail!("pipeline has an empty stage");
            }
            stages.push(Vec::new());
        } else if ch.is_whitespace() {
            if !word.is_empty() {
                stages
                    .last_mut()
                    .expect("stage")
                    .push(std::mem::take(&mut word));
            }
        } else {
            word.push(ch);
        }
    }
    if escaped || quote.is_some() {
        bail!("unterminated quote or escape in saved command");
    }
    if !word.is_empty() {
        stages.last_mut().expect("stage").push(word);
    }
    if stages.last().map_or(true, Vec::is_empty) {
        bail!("pipeline has an empty stage");
    }
    Ok(stages)
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
            command: String::new(),
        };
        assert!(tool.validate().is_ok());

        let resolved = tool.resolve_arguments("example.com", &[]);
        assert_eq!(resolved, vec!["-d", "example.com", "-silent"]);

        let zero_timeout = ToolDefinition {
            timeout_seconds: Some(0),
            ..tool
        };
        assert!(zero_timeout.validate().is_err());
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
            command: String::new(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn parses_quoted_pipeline_without_shell_syntax() {
        let stages = parse_pipeline("echo 'api.{target}' | tool -x \"two words\"").unwrap();
        assert_eq!(
            stages,
            vec![
                vec!["echo", "api.{target}"],
                vec!["tool", "-x", "two words"]
            ]
        );
        assert!(parse_pipeline("echo ok; whoami").is_err());
        assert!(parse_pipeline("echo ok | | cat").is_err());
    }
}
