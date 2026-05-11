use assert_cmd::prelude::*;
use tempfile::TempDir;

#[test]
fn test_intro_first_task_completes() {
    let temp = TempDir::new().expect("failed to create temp dir");

    let mut reset = assert_cmd::cargo::cargo_bin_cmd!("supershell");
    reset
        .env("SUPERSHELL_TEST_MODE", "1")
        .env("XDG_DATA_HOME", temp.path())
        .arg("--reset")
        .arg("--status")
        .assert()
        .success();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("supershell");
    cmd.env("SUPERSHELL_TEST_MODE", "1")
        .env("XDG_DATA_HOME", temp.path())
        .arg("--check")
        .arg("ls")
        .assert()
        .code(2)
        .stdout(predicates::str::contains("[SUCCESS]"))
        .stdout(predicates::str::contains("Sensors Online"));
}
