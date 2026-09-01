use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn init_creates_a_local_repository() {
    let directory = tempdir().expect("temporary directory");
    let mut command = Command::cargo_bin("githunter").expect("githunter binary");

    command
        .current_dir(directory.path())
        .args(["init", "--name", "example.com"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Initialized GitHunter repository.",
        ));

    assert!(directory.path().join(".githunter/config.toml").is_file());
    assert!(directory.path().join(".githunter/githunter.db").is_file());
    assert!(directory
        .path()
        .join(".githunter/metadata/project.json")
        .is_file());
}

#[test]
fn init_without_name_prompts_for_and_saves_initial_setup() {
    let directory = tempdir().expect("temporary directory");
    let mut command = Command::cargo_bin("githunter").expect("githunter binary");

    command
        .current_dir(directory.path())
        .arg("init")
        .write_stdin("prompted-project\nexample.com\nAuthorized program\nexample.com,*.example.com\nadmin.example.com\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Project name"))
        .stdout(predicate::str::contains("Primary target"))
        .stdout(predicate::str::contains("Authorization note"))
        .stdout(predicate::str::contains("In-scope domains"))
        .stdout(predicate::str::contains("Out-of-scope domains"));

    let mut scopes = Command::cargo_bin("githunter").expect("githunter binary");
    scopes
        .current_dir(directory.path())
        .args(["scope", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("IN_SCOPE     example.com"))
        .stdout(predicate::str::contains("IN_SCOPE     *.example.com"))
        .stdout(predicate::str::contains("OUT_OF_SCOPE admin.example.com"));
}

#[test]
fn init_refuses_to_overwrite_an_existing_repository() {
    let directory = tempdir().expect("temporary directory");
    let mut first = Command::cargo_bin("githunter").expect("githunter binary");
    first
        .current_dir(directory.path())
        .arg("init")
        .assert()
        .success();

    let mut second = Command::cargo_bin("githunter").expect("githunter binary");
    second
        .current_dir(directory.path())
        .arg("init")
        .assert()
        .failure();
}
