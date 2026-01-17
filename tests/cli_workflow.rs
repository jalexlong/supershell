use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_full_game_loop() {
    // 1. SETUP: Create a "Clean Room"
    let temp_dir = tempfile::tempdir().unwrap();
    let data_home = temp_dir.path().to_path_buf();

    // Create the library folder
    let library_dir = data_home.join("supershell").join("library");
    fs::create_dir_all(&library_dir).expect("Failed to create library dir");

    // 2. INJECT MOCK QUEST
    // We create a fake '01_awakening.yaml' so the GameState (which defaults to this ID) finds it.
    // This quest requires 'echo hello', which matches our test command.
    let mock_quest = r#"
quests:
  - id: "01_awakening"
    title: "TEST QUEST"
    chapters:
      - title: "Test Chapter"
        intro: "Test Intro"
        outro: "Test Outro"
        tasks:
          - description: "Say Hello"
            instruction: "Just echo hello"
            objective: "echo hello"
            success_msg: "Task 1 Complete"
            conditions:
              - type: CommandMatches
                pattern: "echo hello"
"#;

    let quest_path = library_dir.join("01_awakening.yaml");
    fs::write(&quest_path, mock_quest).expect("Failed to write mock quest");

    // 3. DEBUG: Verify the file exists (Optional, keeps you sane)
    eprintln!("[TEST SETUP] Created mock quest at: {:?}", quest_path);

    // 4. RUN THE TEST
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_supershell"));

    cmd.env("XDG_DATA_HOME", &data_home)
        .arg("--check")
        .arg("echo hello"); // Matches the 'pattern' in our mock quest

    // 5. ASSERT
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Task 1 Complete"));
}
