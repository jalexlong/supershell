use tempfile::TempDir;

fn supershell() -> assert_cmd::Command {
    assert_cmd::cargo::cargo_bin_cmd!("supershell")
}

fn test_env<'a>(cmd: &'a mut assert_cmd::Command, temp: &TempDir) -> &'a mut assert_cmd::Command {
    cmd.env("SUPERSHELL_TEST_MODE", "1")
        .env("XDG_DATA_HOME", temp.path())
}

#[test]
fn reset_and_status_succeeds() {
    let temp = TempDir::new().expect("failed to create temp dir");

    let mut cmd = supershell();
    test_env(&mut cmd, &temp)
        .arg("--reset")
        .arg("--status")
        .assert()
        .success();
}

#[test]
fn irrelevant_command_is_noop() {
    let temp = TempDir::new().expect("failed to create temp dir");

    let mut reset = supershell();
    test_env(&mut reset, &temp)
        .arg("--reset")
        .arg("--status")
        .assert()
        .success();

    let mut cmd = supershell();
    test_env(&mut cmd, &temp)
        .arg("--check")
        .arg("pwd")
        .assert()
        .success()
        .stdout("");
}

#[test]
fn intro_first_task_completes() {
    let temp = TempDir::new().expect("failed to create temp dir");

    let mut reset = supershell();
    test_env(&mut reset, &temp)
        .arg("--reset")
        .arg("--status")
        .assert()
        .success();

    let mut cmd = supershell();
    test_env(&mut cmd, &temp)
        .arg("--check")
        .arg("ls")
        .assert()
        .code(2)
        .stdout(predicates::str::contains("[SUCCESS]"))
        .stdout(predicates::str::contains("Sensors Online"));
}
