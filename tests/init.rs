use assert_cmd::Command;
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
