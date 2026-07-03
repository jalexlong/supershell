# Contributing to SuperShell

First off, thank you for considering contributing! SuperShell is built to make the terminal less intimidating for everyone.

## 🛠 How to Contribute

### 1. Curriculum Development (No Coding Required)
The core content resides in the `library/` directory. You can contribute by:
- Creating new Quest YAML files to teach specific tools (e.g., `grep`, `sed`, `ssh`).
- Fixing typos in existing `intro` or `outro` text.
- Improving Regex patterns in `conditions` to be more forgiving or more precise.

### 2. Core Engine Development
If you want to touch the Rust code, we follow these principles:
- **Data-Driven:** Avoid hardcoding lesson logic in Rust. If it can be a property in YAML, it should be.
- **Scannability:** Keep the UI clean. Use the `ui.rs` module for any text formatting.
- **Platform Agnostic:** We use `ProjectDirs` to ensure this works on Linux, macOS, and Windows.

## 🧪 Testing Your Changes

### Automated Testing (Recommended)

Run the standard quality gate before committing:

```bash
cargo fmt --check && cargo check && cargo test
```

Integration tests live in `tests/cli_workflow.rs`. They use two env vars to isolate save data:

```rust
cmd.env("SUPERSHELL_TEST_MODE", "1")
   .env("XDG_DATA_HOME", temp.path())
```

The mock quest fixture at `tests/fixtures/mock_quest.yaml` provides a minimal two-chapter quest that exercises `CommandMatches`, `FlagIsTrue`, `SetFlag` rewards, `PathExists`, and hint text. Use it (via the `setup_mock_quest` / `setup_mock_quest_at_task2` helpers in the test file) when writing new engine tests rather than coupling tests to the real `intro.yaml` content.

For tests that verify **state persistence across invocations**, use two separate `Command` calls against the same `TempDir`:

```rust
let temp = TempDir::new().unwrap();
setup_mock_quest(&temp);

// First invocation — complete a task
test_env(&mut supershell(), &temp).arg("--check").arg("echo alpha").assert().code(2);

// Second invocation — task-1 command is now irrelevant (state persisted)
test_env(&mut supershell(), &temp).arg("--check").arg("echo alpha").assert().code(0).stdout("");
```

### Manual Testing
To test a new curriculum path you are writing:
1. Save your YAML file to the `library/` folder.
2. Run `cargo run -- --validate library/your_module.yaml` to check YAML syntax.
3. Run `cargo run` to enter the Construct.
4. Walk through your new tasks to ensure the `conditions` trigger correctly.

See `docs/playtesting.md` for the full manual test checklist.

## 📝 Commit Messages
We follow a simple convention for commits:
- `feat:` for new engine capabilities.
- `lesson:` for updates to the YAML curriculum.
- `fix:` for bug fixes.
- `docs:` for documentation updates.
