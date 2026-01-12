use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;

#[test]
fn test_full_game_loop() {
    // 1. SETUP: Create a sandbox environment
    let temp = tempfile::tempdir().unwrap();
    let data_dir = temp.path().join("supershell");
    fs::create_dir_all(&data_dir).unwrap();

    // Create a dummy quests.yaml
    let yaml_path = data_dir.join("quests.yaml");
    let yaml_content = r#"
quests:
  - id: "test_quest"
    title: "Test Quest"
    chapters:
      - title: "Chapter 1"
        intro: "Intro text"
        outro: "Outro text"
        tasks:
          - description: "Task 1"
            instruction: "Do echo"
            objective: "Run echo"
            success_msg: "Task 1 Complete"
            conditions:
              - type: CommandMatches
                pattern: "^echo"
          - description: "Task 2"
            instruction: "Do nothing"
            objective: "Wait"
            success_msg: "Done"
            conditions: []
    "#;
    fs::write(&yaml_path, yaml_content).unwrap();

    // Create a dummy save file so --reset works
    let save_path = data_dir.join("save.json");
    fs::write(&save_path, r#"{"current_quest_id": "test_quest"}"#).unwrap();

    // 2. ENV MOCKING
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_supershell"));
    cmd.env("XDG_DATA_HOME", temp.path().to_str().unwrap());

    // 3. TEST: RESET
    cmd.arg("--reset")
        .assert()
        .success()
        .stdout(predicate::str::contains("Save state wiped"));

    // 4. TEST: CHECK FAIL
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_supershell"));
    cmd.env("XDG_DATA_HOME", temp.path().to_str().unwrap())
        .args(&["--check", "ls"])
        .assert()
        .success();

    // 5. TEST: CHECK SUCCESS
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_supershell"));
    cmd.env("XDG_DATA_HOME", temp.path().to_str().unwrap())
        .args(&["--check", "echo hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Task 1 Complete"));

    // 6. VERIFY PERSISTENCE (ROBUST VERSION)
    let save_content = fs::read_to_string(save_path).unwrap();

    // Parse the JSON instead of string matching
    let v: Value = serde_json::from_str(&save_content).expect("Failed to parse save.json");

    // Check the actual integer value
    // "current_task_index" might be missing if default is 0,
    // but here we expect it to be 1.
    let task_index = v["current_task_index"].as_u64().unwrap_or(0);

    assert_eq!(
        task_index, 1,
        "Task index should have incremented to 1. Current JSON: {}",
        save_content
    );
}
