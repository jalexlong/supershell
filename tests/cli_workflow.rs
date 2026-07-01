use std::fs;
use tempfile::TempDir;

fn supershell() -> assert_cmd::Command {
    assert_cmd::cargo::cargo_bin_cmd!("supershell")
}

fn test_env<'a>(cmd: &'a mut assert_cmd::Command, temp: &TempDir) -> &'a mut assert_cmd::Command {
    cmd.env("SUPERSHELL_TEST_MODE", "1")
        .env("XDG_DATA_HOME", temp.path())
}

/// Copy the mock quest fixture into the test library dir and write a save file
/// pointing to it at chapter 1, task 1 (initial state).
fn setup_mock_quest(temp: &TempDir) {
    let lib_dir = temp.path().join("supershell").join("library");
    fs::create_dir_all(&lib_dir).expect("failed to create library dir");
    fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/mock_quest.yaml"
        ),
        lib_dir.join("mock_quest.yaml"),
    )
    .expect("failed to copy mock_quest.yaml fixture");

    fs::write(
        temp.path().join("supershell").join("save.json"),
        r#"{"current_course":"mock_quest.yaml","course_version":"0.1.0","current_quest_id":"mock","current_chapter_index":0,"current_task_index":0,"flags":{},"variables":{},"is_finished":false}"#,
    )
    .expect("failed to write save.json");
}

/// Same as setup_mock_quest but places the game at chapter 1, task 2 (index 1)
/// with no flags set — task 2 requires FlagIsTrue(alpha_done) to pass.
fn setup_mock_quest_at_task2(temp: &TempDir) {
    let lib_dir = temp.path().join("supershell").join("library");
    fs::create_dir_all(&lib_dir).expect("failed to create library dir");
    fs::copy(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/mock_quest.yaml"
        ),
        lib_dir.join("mock_quest.yaml"),
    )
    .expect("failed to copy mock_quest.yaml fixture");

    fs::write(
        temp.path().join("supershell").join("save.json"),
        r#"{"current_course":"mock_quest.yaml","course_version":"0.1.0","current_quest_id":"mock","current_chapter_index":0,"current_task_index":1,"flags":{},"variables":{},"is_finished":false}"#,
    )
    .expect("failed to write save.json");
}

// ── Existing tests ────────────────────────────────────────────────────────────

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
        .arg("cd Memory_Bank")
        .arg("--cwd")
        .arg(memory_bank_cwd)
        .assert()
        .code(2)
        .stdout(predicates::str::contains("Transfer complete"));
}

// ── M2 tests (mock quest fixture) ────────────────────────────────────────────

/// Completing a task in one process invocation must persist to disk so that
/// the next invocation sees the updated state. Verified by checking that the
/// task-1 command becomes irrelevant (exit 0, empty stdout) after task-1 was
/// already completed in a prior invocation.
#[test]
fn state_persists_across_invocations() {
    let temp = TempDir::new().expect("failed to create temp dir");
    setup_mock_quest(&temp);

    // First invocation: complete task 1.
    let mut first = supershell();
    test_env(&mut first, &temp)
        .arg("--check")
        .arg("echo alpha")
        .assert()
        .code(2)
        .stdout(predicates::str::contains("Alpha acknowledged"));

    // Second invocation: task 1's command is now irrelevant (we are on task 2).
    let mut second = supershell();
    test_env(&mut second, &temp)
        .arg("--check")
        .arg("echo alpha")
        .assert()
        .code(0)
        .stdout("");
}

/// A command that matches the relevance pattern but fails a logic condition
/// (FlagIsTrue not set) must return exit code 1 and print the failure message.
#[test]
fn failure_returns_exit_code_1() {
    let temp = TempDir::new().expect("failed to create temp dir");
    setup_mock_quest_at_task2(&temp); // task 2 active, alpha_done flag absent

    let mut cmd = supershell();
    test_env(&mut cmd, &temp)
        .arg("--check")
        .arg("echo beta")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("Alpha must be completed first."));
}

/// --refresh after --reset must succeed without panicking and must print
/// mission content (cutscene + status card).
#[test]
fn refresh_succeeds_after_reset() {
    let temp = TempDir::new().expect("failed to create temp dir");

    let mut reset = supershell();
    test_env(&mut reset, &temp)
        .arg("--reset")
        .arg("--status")
        .assert()
        .success();

    let mut refresh = supershell();
    test_env(&mut refresh, &temp)
        .arg("--refresh")
        .assert()
        .success()
        .stdout(predicates::str::contains("MISSION BRIEF"));
}

/// Completing two tasks in sequence must advance the game through both tasks
/// in chapter 1 and into chapter 2.
#[test]
fn multi_task_progression() {
    let temp = TempDir::new().expect("failed to create temp dir");
    setup_mock_quest(&temp);

    // Task 1: sets the alpha_done flag.
    let mut t1 = supershell();
    test_env(&mut t1, &temp)
        .arg("--check")
        .arg("echo alpha")
        .assert()
        .code(2)
        .stdout(predicates::str::contains("Alpha acknowledged"));

    // Task 2: gated by FlagIsTrue(alpha_done) — passes because task 1 set it.
    let mut t2 = supershell();
    test_env(&mut t2, &temp)
        .arg("--check")
        .arg("echo beta")
        .assert()
        .code(2)
        .stdout(predicates::str::contains("Beta acknowledged"));

    // Now in chapter 2. Task-1 command is irrelevant.
    let mut after = supershell();
    test_env(&mut after, &temp)
        .arg("--check")
        .arg("echo alpha")
        .assert()
        .code(0)
        .stdout("");
}

/// Completing the last task of a chapter must trigger the next chapter's
/// setup_actions. Chapter 2 setup creates mock_verify/beacon.txt in ~/Construct;
/// chapter 2 task 1 has PathExists(mock_verify/beacon.txt) as a condition and
/// must succeed (exit 2) once setup has run.
#[test]
fn chapter_transition_triggers_setup() {
    let temp = TempDir::new().expect("failed to create temp dir");
    setup_mock_quest(&temp);

    // Complete chapter 1.
    let mut t1 = supershell();
    test_env(&mut t1, &temp)
        .arg("--check")
        .arg("echo alpha")
        .assert()
        .code(2);

    let mut t2 = supershell();
    test_env(&mut t2, &temp)
        .arg("--check")
        .arg("echo beta")
        .assert()
        .code(2); // chapter transition fires here; chapter 2 setup runs

    // Chapter 2 task 1: PathExists(mock_verify/beacon.txt) must pass.
    let mut t3 = supershell();
    test_env(&mut t3, &temp)
        .arg("--check")
        .arg("echo gamma")
        .assert()
        .code(2)
        .stdout(predicates::str::contains("Gamma acknowledged"));
}

// ── M3 tests (interactive menu) ──────────────────────────────────────────────

/// With a single module available --menu must auto-select it without blocking
/// on interactive input and must print a confirmation before exiting cleanly.
#[test]
fn menu_auto_selects_single_module() {
    let temp = TempDir::new().expect("failed to create temp dir");

    let mut reset = supershell();
    test_env(&mut reset, &temp)
        .arg("--reset")
        .arg("--status")
        .assert()
        .success();

    let mut menu = supershell();
    test_env(&mut menu, &temp)
        .arg("--menu")
        .assert()
        .success()
        .stdout(predicates::str::contains("Auto-selected"))
        .stdout(predicates::str::contains("Module selected"));
}

/// After --menu saves a selection, --status must succeed without requiring
/// the user to run --reset again (regression guard: menu must persist state).
#[test]
fn menu_selection_persists_for_status() {
    let temp = TempDir::new().expect("failed to create temp dir");

    let mut menu = supershell();
    test_env(&mut menu, &temp).arg("--menu").assert().success();

    let mut status = supershell();
    test_env(&mut status, &temp)
        .arg("--status")
        .assert()
        .success();
}

/// SetFlag rewards must persist and enable logic-gated tasks in a later
/// invocation. Without the reward the gated task fails; after the reward it
/// succeeds.
#[test]
fn reward_application_enables_gated_task() {
    // Without completing task 1 first (flag absent), task 2 must fail.
    let temp_no_flag = TempDir::new().expect("failed to create temp dir");
    setup_mock_quest_at_task2(&temp_no_flag);

    let mut without_flag = supershell();
    test_env(&mut without_flag, &temp_no_flag)
        .arg("--check")
        .arg("echo beta")
        .assert()
        .code(1);

    // After completing task 1 (SetFlag reward applied), task 2 must succeed.
    let temp_with_flag = TempDir::new().expect("failed to create temp dir");
    setup_mock_quest(&temp_with_flag);

    let mut set_flag = supershell();
    test_env(&mut set_flag, &temp_with_flag)
        .arg("--check")
        .arg("echo alpha")
        .assert()
        .code(2);

    let mut with_flag = supershell();
    test_env(&mut with_flag, &temp_with_flag)
        .arg("--check")
        .arg("echo beta")
        .assert()
        .code(2);
}
