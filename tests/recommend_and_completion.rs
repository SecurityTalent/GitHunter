use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn githunter() -> Command {
    Command::cargo_bin("githunter").expect("githunter binary")
}

#[test]
fn recommend_provides_state_based_suggestions() {
    let dir = tempdir().expect("tempdir");

    githunter()
        .current_dir(dir.path())
        .args(["init", "--name", "rec-test"])
        .assert()
        .success();

    // Recommendation with fresh repo
    githunter()
        .current_dir(dir.path())
        .args(["recommend"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "GITHUNTER RESEARCH RECOMMENDATIONS",
        ))
        .stdout(predicate::str::contains(
            "Register primary authorized target",
        ))
        .stdout(predicate::str::contains("Define In-Scope rules"));

    // Add target and scope, then check recommendation evolves
    githunter()
        .current_dir(dir.path())
        .args(["target", "add", "target.com"])
        .assert()
        .success();

    githunter()
        .current_dir(dir.path())
        .args(["scope", "add", "*.target.com"])
        .assert()
        .success();

    githunter()
        .current_dir(dir.path())
        .args(["recommend"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Ingest initial recon assets"));
}

#[test]
fn generates_shell_completions() {
    githunter()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_githunter"));

    githunter()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Register-ArgumentCompleter"));
}
