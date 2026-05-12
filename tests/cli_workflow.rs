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

#[test]
fn validate_intro_module_succeeds() {
    let mut cmd = supershell();

    cmd.arg("--validate")
        .arg("library/intro.yaml")
        .assert()
        .success()
        .stdout(predicates::str::contains("[SUCCESS]"))
        .stdout(predicates::str::contains("YAML Syntax is valid"))
        .stdout(predicates::str::contains("Version:"));
}

#[test]
fn cd_task_uses_explicit_cwd() {
    let temp = TempDir::new().expect("failed to create temp dir");

    let mut reset = supershell();
    test_env(&mut reset, &temp)
        .arg("--reset")
        .arg("--status")
        .assert()
        .success();

    let mut complete_first_task = supershell();
    test_env(&mut complete_first_task, &temp)
        .arg("--check")
        .arg("ls")
        .assert()
        .code(2)
        .stdout(predicates::str::contains("Sensors Online"));

    let memory_bank_cwd = temp.path().join("Construct").join("Memory_Bank");

    let mut complete_cd_task = supershell();
    test_env(&mut complete_cd_task, &temp)
        .arg("--check")
        .arg("cd Memory_Bank/")
        .arg("--cwd")
        .arg(memory_bank_cwd)
        .arg("--command-status")
        .arg("0")
        .assert()
        .code(2)
        .stdout(predicates::str::contains("Transfer complete"));
}

#[test]
fn failed_cd_command_does_not_complete_task() {
    let temp = TempDir::new().expect("failed to create temp dir");

    let mut reset = supershell();
    test_env(&mut reset, &temp)
        .arg("--reset")
        .arg("--status")
        .assert()
        .success();

    let mut complete_first_task = supershell();
    test_env(&mut complete_first_task, &temp)
        .arg("--check")
        .arg("ls")
        .assert()
        .code(2)
        .stdout(predicates::str::contains("Sensors Online"));

    let memory_bank_cwd = temp.path().join("Construct").join("Memory_Bank");

    let mut failed_cd_task = supershell();
    test_env(&mut failed_cd_task, &temp)
        .arg("--check")
        .arg("cd Memory_Bank")
        .arg("--cwd")
        .arg(memory_bank_cwd)
        .arg("--command-status")
        .arg("1")
        .assert()
        .success()
        .stdout(predicates::str::is_empty());

    let mut status = supershell();
    test_env(&mut status, &temp)
        .arg("--status")
        .assert()
        .success()
        .stdout(predicates::str::contains("Motor Functions"))
        .stdout(predicates::str::contains("Enter the Memory Bank"));
}
