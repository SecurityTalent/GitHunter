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
        .args(["scope", "add", "target.com"])
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

    githunter()
        .current_dir(dir.path())
        .args(["asset", "add", "target.com", "--source", "manual-test"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Added asset: target.com (DOMAIN, IN_SCOPE)",
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

    // Positional `all` means no scope filter; the following number limits rows.
    githunter()
        .current_dir(dir.path())
        .args(["asset", "list", "--type", "domain", "all", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("target.com"));

    // `asset` without an operation is a concise alias for `asset list`.
    githunter()
        .current_dir(dir.path())
        .args(["asset", "--type", "subdomain", "--scope", "in_scope"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api.target.com"))
        .stdout(predicate::str::contains("dev.target.com"));

    githunter()
        .current_dir(dir.path())
        .args(["asset", "--source", "subfinder", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("subfinder"));

    // Status summarizes the persisted assets (not just the most recent import).
    githunter()
        .current_dir(dir.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Assets: 9"))
        .stdout(predicate::str::contains("Types:"))
        .stdout(predicate::str::contains("SUBDOMAIN"))
        .stdout(predicate::str::contains("Scope:"))
        .stdout(predicate::str::contains("IN_SCOPE"))
        .stdout(predicate::str::contains("OUT_OF_SCOPE"));

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

#[test]
fn asn_and_cidr_are_normalized_and_deduplicated() {
    let dir = tempdir().expect("tempdir");
    githunter()
        .current_dir(dir.path())
        .args(["init", "--name", "network-assets"])
        .assert()
        .success();
    githunter()
        .current_dir(dir.path())
        .args(["asset", "add", "AS0013335"])
        .assert()
        .success()
        .stdout(predicate::str::contains("AS13335 (ASN"));
    githunter()
        .current_dir(dir.path())
        .args(["asset", "add", "192.168.1.99/24"])
        .assert()
        .success()
        .stdout(predicate::str::contains("192.168.1.0/24 (CIDR"));
    githunter()
        .current_dir(dir.path())
        .args(["asset", "add", "192.168.1.0/24"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Asset already tracked"));
    githunter()
        .current_dir(dir.path())
        .args(["asset", "list", "--type", "asn"])
        .assert()
        .success()
        .stdout(predicate::str::contains("AS13335"));
    githunter()
        .current_dir(dir.path())
        .args(["scope", "add", "AS13335"])
        .assert()
        .success();
    githunter()
        .current_dir(dir.path())
        .args(["scope", "check", "AS13335"])
        .assert()
        .success()
        .stdout(predicate::str::contains("IN_SCOPE"));
}

#[test]
fn asset_export_is_clean_pipeline_output() {
    let dir = tempdir().expect("tempdir");
    githunter()
        .current_dir(dir.path())
        .args(["init", "--name", "export-test"])
        .assert()
        .success();
    githunter()
        .current_dir(dir.path())
        .args(["asset", "add", "one.example.com"])
        .assert()
        .success();
    githunter()
        .current_dir(dir.path())
        .args(["asset", "add", "two.example.com"])
        .assert()
        .success();
    githunter()
        .current_dir(dir.path())
        .args(["asset", "export", "--type", "subdomain"])
        .assert()
        .success()
        .stdout("one.example.com\ntwo.example.com\n");
}

#[test]
fn requested_asset_list_export_and_httpx_import_workflow() {
    let dir = tempdir().expect("tempdir");
    githunter()
        .current_dir(dir.path())
        .args(["init", "--name", "pipeline-test"])
        .assert()
        .success();
    for rule in ["example.com", "*.example.com"] {
        githunter()
            .current_dir(dir.path())
            .args(["scope", "add", rule])
            .assert()
            .success();
    }
    for value in ["example.com", "api.example.com", "www.example.com"] {
        githunter()
            .current_dir(dir.path())
            .args(["asset", "add", value, "--source", "manual"])
            .assert()
            .success();
    }

    githunter()
        .current_dir(dir.path())
        .args(["asset", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("api.example.com"));
    githunter()
        .current_dir(dir.path())
        .args(["asset", "list", "--type", "domain", "all", "50"])
        .assert()
        .success()
        .stdout(predicate::str::contains("example.com"));
    githunter()
        .current_dir(dir.path())
        .args([
            "asset",
            "export",
            "--type",
            "subdomain",
            "--scope",
            "in_scope",
        ])
        .assert()
        .success()
        .stdout("api.example.com\nwww.example.com\n");

    // This is the final stage of `asset export | httpx -silent | asset import -`.
    // A URL is representative of httpx's silent output and proves stdin ingestion.
    githunter()
        .current_dir(dir.path())
        .args(["asset", "import", "-", "--source", "httpx"])
        .write_stdin("https://api.example.com\nhttps://www.example.com\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("New assets: 2"));
    githunter()
        .current_dir(dir.path())
        .args(["asset", "list", "--source", "httpx"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://api.example.com"))
        .stdout(predicate::str::contains("https://www.example.com"));
}

#[test]
fn asset_list_all_accepts_a_user_selected_limit() {
    let dir = tempdir().expect("tempdir");
    githunter()
        .current_dir(dir.path())
        .args(["init", "--name", "asset-list-limit-test"])
        .assert()
        .success();

    for value in ["alpha.example", "bravo.example", "charlie.example"] {
        githunter()
            .current_dir(dir.path())
            .args(["asset", "add", value])
            .assert()
            .success();
    }

    // `all` removes the scope filter, and the next positional value is the
    // caller-selected maximum number of rows to print.
    let output = githunter()
        .current_dir(dir.path())
        .args(["asset", "list", "--type", "domain", "all", "2", "--json"])
        .output()
        .expect("asset list command should run");
    assert!(output.status.success());

    let listed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("asset list should emit JSON");
    assert_eq!(
        listed
            .as_array()
            .expect("list output should be an array")
            .len(),
        2
    );
}
