use assert_cmd::Command;
use tempfile::tempdir;

fn githunter() -> Command {
    Command::cargo_bin("githunter").expect("githunter binary")
}

#[test]
fn tracks_authorized_assets_and_snapshot_changes() {
    let directory = tempdir().expect("temporary directory");
    let input = directory.path().join("assets.txt");
    std::fs::write(&input, "api.example.com\nhttps://example.com/login\n").expect("asset input");

    githunter()
        .current_dir(directory.path())
        .args(["init", "--name", "example.com"])
        .assert()
        .success();
    githunter()
        .current_dir(directory.path())
        .args(["target", "add", "example.com"])
        .assert()
        .success();
    githunter()
        .current_dir(directory.path())
        .args(["scope", "add", "*.example.com"])
        .assert()
        .success();
    githunter()
        .current_dir(directory.path())
        .args(["asset", "import", "assets.txt", "--source", "manual"])
        .assert()
        .success()
        .stdout(predicates::str::contains("New assets: 2"));
    githunter()
        .current_dir(directory.path())
        .args(["snapshot", "create", "--note", "baseline"])
        .assert()
        .success()
        .stdout(predicates::str::contains("s_0001"));

    std::fs::write(&input, "staging.example.com\n").expect("second asset input");
    githunter()
        .current_dir(directory.path())
        .args(["asset", "import", "assets.txt"])
        .assert()
        .success();
    githunter()
        .current_dir(directory.path())
        .args(["snapshot", "create"])
        .assert()
        .success();
    githunter()
        .current_dir(directory.path())
        .arg("diff")
        .assert()
        .success()
        .stdout(predicates::str::contains("Added: 1"));
}
