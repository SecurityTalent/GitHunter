use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn githunter() -> Command {
    Command::cargo_bin("githunter").expect("githunter binary")
}

#[test]
fn tool_and_workflow_management() {
    let dir = tempdir().expect("tempdir");

    githunter()
        .current_dir(dir.path())
        .args(["init", "--name", "tools-test"])
        .assert()
        .success();

    // 1. Add a tool
    githunter()
        .current_dir(dir.path())
        .args([
            "tool",
            "add",
            "--name",
            "mock-echo",
            "--executable",
            "cmd",
            "--description",
            "Mock echo tool for testing",
            "--args",
            "/c echo api.{target}",
            "--tags",
            "subdomain-discovery,passive",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configured tool: mock-echo"));

    // 2. List tools
    githunter()
        .current_dir(dir.path())
        .args(["tool", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mock-echo"))
        .stdout(predicate::str::contains("subdomain-discovery"));

    // 3. Show tool
    githunter()
        .current_dir(dir.path())
        .args(["tool", "show", "mock-echo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Mock echo tool for testing"));

    // 4. Validate tool
    githunter()
        .current_dir(dir.path())
        .args(["tool", "validate", "mock-echo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration syntax valid"));

    // 5. Add target and run tool with automated output ingestion
    githunter()
        .current_dir(dir.path())
        .args(["target", "add", "example.com"])
        .assert()
        .success();

    githunter()
        .current_dir(dir.path())
        .args(["tool", "run", "mock-echo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Running 'mock-echo'"))
        .stdout(predicate::str::contains(
            "Ingesting tool output into GitHunter asset pipeline",
        ))
        .stdout(predicate::str::contains("New assets: 1"));

    // Check that asset was ingested
    githunter()
        .current_dir(dir.path())
        .args(["asset", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api.example.com"));

    // 6. Workflow management
    githunter()
        .current_dir(dir.path())
        .args([
            "workflow",
            "add",
            "--name",
            "recon-flow",
            "--description",
            "Automated reconnaissance flow",
            "--steps",
            "mock-echo",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configured workflow: recon-flow"));

    githunter()
        .current_dir(dir.path())
        .args(["workflow", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("recon-flow"));

    githunter()
        .current_dir(dir.path())
        .args(["workflow", "run", "recon-flow"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Workflow 'recon-flow' completed successfully",
        ));

    // 7. Remove tool
    githunter()
        .current_dir(dir.path())
        .args(["tool", "remove", "mock-echo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed tool: mock-echo"));
}
