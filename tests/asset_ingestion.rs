use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn githunter() -> Command {
    Command::cargo_bin("githunter").expect("githunter binary")
}

#[test]
fn mixed_asset_types_and_deduplication_workflow() {
    let dir = tempdir().expect("tempdir");

    githunter()
        .current_dir(dir.path())
        .args(["init", "--name", "assets-test"])
        .assert()
        .success();

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
        .args(["scope", "out", "add", "admin.target.com"])
        .assert()
        .success();

    // 1. Test single asset add
    githunter()
        .current_dir(dir.path())
        .args(["asset", "add", "api.target.com", "--source", "manual-test"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Added asset: api.target.com (SUBDOMAIN, IN_SCOPE)",
        ));

    // 2. Test mixed asset file import
    let assets_file = dir.path().join("mixed_assets.txt");
    std::fs::write(
        &assets_file,
        "# Mixed Recon Assets\n\
         API.TARGET.COM\n\
         dev.target.com\n\
         https://target.com/login\n\
         https://admin.target.com:443/dashboard\n\
         192.168.1.10\n\
         192.168.1.20:8080\n\
         /api/v1/users\n\
         otherdomain.org\n",
    )
    .expect("write assets");

    githunter()
        .current_dir(dir.path())
        .args([
            "asset",
            "import",
            assets_file.to_str().unwrap(),
            "--source",
            "subfinder",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported: 8"))
        .stdout(predicate::str::contains("New assets: 7"))
        .stdout(predicate::str::contains("Duplicates: 1"))
        .stdout(predicate::str::contains("SUBDOMAIN"))
        .stdout(predicate::str::contains("URL"))
        .stdout(predicate::str::contains("IP"))
        .stdout(predicate::str::contains("ENDPOINT"))
        .stdout(predicate::str::contains("IN_SCOPE"))
        .stdout(predicate::str::contains("OUT_OF_SCOPE"));

    // 3. Test multiple sources / provenance tracking
    // Re-importing same assets from "amass" should associate "amass" source without adding duplicate assets
    githunter()
        .current_dir(dir.path())
        .args([
            "asset",
            "import",
            assets_file.to_str().unwrap(),
            "--source",
            "amass",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("New assets: 0"))
        .stdout(predicate::str::contains("Duplicates: 8"));

    // 4. Test asset list JSON output with multiple sources
    githunter()
        .current_dir(dir.path())
        .args(["asset", "list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api.target.com"))
        .stdout(predicate::str::contains("subfinder"))
        .stdout(predicate::str::contains("amass"));

    // 5. Test asset list filtering by type and scope
    githunter()
        .current_dir(dir.path())
        .args(["asset", "list", "--type", "subdomain"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api.target.com"))
        .stdout(predicate::str::contains("dev.target.com"));

    githunter()
        .current_dir(dir.path())
        .args(["asset", "list", "--scope", "out_of_scope"])
        .assert()
        .success()
        .stdout(predicate::str::contains("admin.target.com"));
}

#[test]
fn stdin_asset_ingestion() {
    let dir = tempdir().expect("tempdir");

    githunter()
        .current_dir(dir.path())
        .args(["init", "--name", "stdin-test"])
        .assert()
        .success();

    // Ingest via stdin explicitly with "-"
    githunter()
        .current_dir(dir.path())
        .args(["asset", "import", "-", "--source", "httpx-pipe"])
        .write_stdin("auth.example.com\nhttps://example.com/api\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Imported: 2"))
        .stdout(predicate::str::contains("New assets: 2"));
}
