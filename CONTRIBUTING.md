# Contributing to SuperShell

First off, thank you for considering contributing! SuperShell is built to make the terminal less intimidating for everyone.

## 🛠 How to Contribute

### 1. Curriculum Development (No Coding Required)
The core of this project is the `quests.yaml` file. You can contribute by:
- Fixing typos in existing briefings.
- Adding new Chapters to cover tools like `grep`, `sed`, or `ssh`.
- Improving Regex patterns in `conditions` to be more forgiving or more precise.

### 2. Core Engine Development
If you want to touch the Rust code, we follow these principles:
- **Data-Driven:** Avoid hardcoding lesson logic in Rust. If it can be a property in YAML, it should be.
- **Scannability:** Keep the UI clean. Use the `ui.rs` module for any text formatting.
- **Platform Agnostic:** We use `ProjectDirs` to ensure this works on Linux, macOS, and Windows.

## 🧪 Testing Your Changes
To test a new curriculum path:
1. Update `quests.yaml`.
2. Run `cargo build`.
3. Use `supershell --reset` to verify the "Inherent Beginning" discovery.
4. Walk through your new checkpoints to ensure the `conditions` trigger correctly.

## 📝 Commit Messages
We follow a simple convention for commits:
- `feat:` for new engine capabilities.
- `lesson:` for updates to the YAML curriculum.
- `fix:` for bug fixes.
- `docs:` for documentation updates.
