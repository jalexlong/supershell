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
We use a **Mock Quest** strategy for integration testing. This allows us to test the engine logic without relying on the actual story content.

1. Run `cargo test`.
2. The tests will spin up a temporary directory, inject a mock quest, and verify the game loop handles inputs correctly.

### Manual Testing
To test a new curriculum path you are writing:
1. Save your YAML file to the `library/` folder.
2. Run `cargo run`.
3. The engine will auto-detect the new file.
4. Walk through your new tasks to ensure the `conditions` trigger correctly.

## 📝 Commit Messages
We follow a simple convention for commits:
- `feat:` for new engine capabilities.
- `lesson:` for updates to the YAML curriculum.
- `fix:` for bug fixes.
- `docs:` for documentation updates.
