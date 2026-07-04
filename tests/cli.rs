use std::{fs, path::Path};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

fn plus() -> Command {
    Command::cargo_bin("plus").expect("plus binary")
}

fn cargo_plus() -> Command {
    Command::cargo_bin("cargo-plus").expect("cargo-plus binary")
}

fn fixture() -> TempDir {
    let temp = TempDir::new().expect("temp dir");
    fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::write(temp.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    temp
}

fn write_file(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, bytes).unwrap();
}

#[test]
fn init_creates_plus_toml_without_overwriting() {
    let temp = fixture();

    plus()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("created"));

    let config = temp.path().join("plus.toml");
    assert!(config.exists());

    fs::write(&config, "sentinel = true\n").unwrap();
    plus()
        .current_dir(temp.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));
    assert_eq!(fs::read_to_string(&config).unwrap(), "sentinel = true\n");

    plus()
        .current_dir(temp.path())
        .args(["init", "--force"])
        .assert()
        .success();
    assert!(fs::read_to_string(&config).unwrap().contains("[commands]"));
}

#[test]
fn size_reports_deep_target_buckets() {
    let temp = fixture();
    write_file(
        &temp.path().join("target/debug/deps/libfixture.rlib"),
        b"dependency artifact",
    );
    write_file(
        &temp.path().join("target/debug/incremental/cache/file"),
        b"incremental artifact",
    );

    plus()
        .current_dir(temp.path())
        .args(["size", "--deep"])
        .assert()
        .success()
        .stdout(predicate::str::contains("target:"))
        .stdout(predicate::str::contains("debug/deps"))
        .stdout(predicate::str::contains("debug/incremental"));
}

#[test]
fn size_json_is_machine_readable() {
    let temp = fixture();
    write_file(
        &temp.path().join("target/debug/incremental/cache/file"),
        b"incremental artifact",
    );

    let output = plus()
        .current_dir(temp.path())
        .args(["size", "--deep", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert!(json["total_bytes"].as_u64().unwrap() > 0);
    assert!(json["deep"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["path"].as_str().unwrap().contains("incremental")));
}

#[test]
fn clean_defaults_to_dry_run() {
    let temp = fixture();
    let incremental = temp.path().join("target/debug/incremental/cache/file");
    write_file(&incremental, b"keep until apply");

    plus()
        .current_dir(temp.path())
        .arg("clean")
        .assert()
        .success()
        .stdout(predicate::str::contains("previewing clean"));

    assert!(incremental.exists());
}

#[test]
fn clean_json_reports_plan_without_deleting() {
    let temp = fixture();
    let incremental = temp.path().join("target/debug/incremental/cache/file");
    write_file(&incremental, b"keep until apply");

    let output = plus()
        .current_dir(temp.path())
        .args(["clean", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["apply"], false);
    assert!(json["total_bytes"].as_u64().unwrap() > 0);
    assert!(incremental.exists());
}

#[test]
fn clean_apply_removes_safe_items_only() {
    let temp = fixture();
    let incremental = temp.path().join("target/debug/incremental/cache/file");
    let deps = temp.path().join("target/debug/deps/libfixture.rlib");
    write_file(&incremental, b"remove me");
    write_file(&deps, b"keep me");

    plus()
        .current_dir(temp.path())
        .args(["clean", "--apply"])
        .assert()
        .success()
        .stdout(predicate::str::contains("freed"));

    assert!(!temp.path().join("target/debug/incremental").exists());
    assert!(deps.exists());
}

#[test]
fn clean_deep_apply_removes_build_outputs() {
    let temp = fixture();
    let build = temp.path().join("target/debug/build/build-script/out");
    write_file(&build, b"generated");

    plus()
        .current_dir(temp.path())
        .args(["clean", "--deep", "--apply"])
        .assert()
        .success();

    assert!(!temp.path().join("target/debug/build").exists());
}

#[test]
fn nuclear_clean_requires_yes() {
    let temp = fixture();

    plus()
        .current_dir(temp.path())
        .args(["clean", "--nuclear"])
        .assert()
        .failure()
        .stdout(predicate::str::contains("--nuclear --yes"));
}

#[test]
fn run_uses_custom_plus_toml_command() {
    let temp = fixture();
    fs::write(
        temp.path().join("plus.toml"),
        "[commands]\nhello = \"cargo --version\"\n",
    )
    .unwrap();

    plus()
        .current_dir(temp.path())
        .args(["run", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cargo "));
}

#[test]
fn doctor_json_is_machine_readable() {
    let temp = fixture();

    let output = plus()
        .current_dir(temp.path())
        .args(["doctor", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(
        json["project"]["manifest"].as_str().unwrap(),
        temp.path().join("Cargo.toml").to_str().unwrap()
    );
    assert!(json["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|tool| tool["name"] == "cargo"));
    assert!(json["recommendations"].is_array());
}

#[test]
fn cargo_plus_accepts_cargo_subcommand_prefix() {
    let temp = fixture();

    let output = cargo_plus()
        .current_dir(temp.path())
        .args(["plus", "doctor", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(
        json["project"]["manifest"].as_str().unwrap(),
        temp.path().join("Cargo.toml").to_str().unwrap()
    );
}
