use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn githunter() -> Command {
    Command::cargo_bin("githunter").expect("githunter binary")
}

#[test]
fn scope_file_import_and_deduplication() {
    let dir = tempdir().expect("tempdir");

    // Init repository
    githunter()
        .current_dir(dir.path())
        .args(["init", "--name", "scope-test"])
        .assert()
        .success();

    // Create scope file with comments, whitespace, and duplicates
    let scope_file = dir.path().join("scope.txt");
    std::fs::write(
        &scope_file,
        "# Primary scope definitions\n*.target.com\ntarget.com\n  API.TARGET.COM  \n# Duplicates:\n*.target.com\ntarget.com.\n",
    )
    .expect("write scope file");

    // Import scope from file
    githunter()
        .current_dir(dir.path())
        .args(["scope", "add", "--file", scope_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added: 3"))
        .stdout(predicate::str::contains("Skipped duplicates: 2"));

    // Re-importing same file should result in 0 added, 3 duplicates skipped
    githunter()
        .current_dir(dir.path())
        .args(["scope", "add", "--file", scope_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added: 0"))
        .stdout(predicate::str::contains("Skipped duplicates: 5"));

    // Create out-of-scope file
    let outscope_file = dir.path().join("outscope.txt");
    std::fs::write(
        &outscope_file,
        "admin.target.com\ninternal.target.com # internal admin portal\n",
    )
    .expect("write outscope file");

    githunter()
        .current_dir(dir.path())
        .args([
            "scope",
            "out",
            "add",
            "--file",
            outscope_file.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added: 2"))
        .stdout(predicate::str::contains("Skipped duplicates: 0"));

    // Verify scope check precedence: OUT_OF_SCOPE beats IN_SCOPE
    githunter()
        .current_dir(dir.path())
        .args(["scope", "check", "api.target.com"])
        .assert()
        .success()
        .stdout(predicate::str::contains("IN_SCOPE"));

    githunter()
        .current_dir(dir.path())
        .args(["scope", "check", "admin.target.com"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OUT_OF_SCOPE"));

    githunter()
        .current_dir(dir.path())
        .args(["scope", "check", "https://admin.target.com/login"])
        .assert()
        .success()
        .stdout(predicate::str::contains("OUT_OF_SCOPE"));

    githunter()
        .current_dir(dir.path())
        .args(["scope", "check", "unrelated-domain.com"])
        .assert()
        .success()
        .stdout(predicate::str::contains("UNKNOWN"));
}

#[test]
fn single_scope_rule_deduplication() {
    let dir = tempdir().expect("tempdir");

    githunter()
        .current_dir(dir.path())
        .args(["init"])
        .assert()
        .success();

    // First add
    githunter()
        .current_dir(dir.path())
        .args(["scope", "add", "*.example.com"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Added IN_SCOPE: *.example.com"));

    // Normalized duplicate add
    githunter()
        .current_dir(dir.path())
        .args(["scope", "add", " *.EXAMPLE.COM. "])
        .assert()
        .success()
        .stdout(predicate::str::contains("Skipped duplicate: *.example.com"));
}
