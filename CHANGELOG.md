# Changelog

## 📜 Release History

# v0.4.1: The "Self-Extracting" Update

**Summary:**
This release radically simplifies installation. The `supershell` binary now carries the entire game library inside itself. When you run it for the first time, it automatically installs the necessary assets to your system.

**New Features:**
* 📦 **Self-Extracting Binary:** The `library/` folder is now embedded in the executable. No external installation scripts are required.
* 🚀 **Crates.io Support:** You can now install the full game with a single command: `cargo install supershell`.
* 🛠️ **Auto-Repair:** If the game detects the `library` folder is missing (e.g., accidental deletion), it will automatically restore the default quests on the next launch.

**Changes:**
* Removed `install.sh` and `uninstall.sh` (deprecated).
* Simplified distribution artifacts to just the binary and README.
* Updated internal path resolution to handle embedded asset extraction.

## v0.4.0 - The Stability Update

* **Universal YAML Loader:** The engine now intelligently parses quest files in multiple formats (Wrapped Object, List, or Single Object), preventing crashes when loading user-created content.
* **Graceful Degradation:** Optional fields in Quest/Chapter definitions (like `setup_actions`) are now handled gracefully. Missing fields no longer cause the application to panic or fail silently.
* **Startup Fixes:** Resolved an issue where a fresh installation would initialize with an empty Quest ID, causing the game to boot into a "do nothing" state. The default is now set to `01_awakening`.
* **Error Visibility:** File system errors (permissions, missing directories) during the library scan are now logged to stderr as `[CRITICAL ERROR]` instead of being swallowed silently.

### v0.3.1 (The Distribution Patch)
- **Fixed:** Critical installer bug on macOS regarding `Read-only file system` errors.
- **Fixed:** Improved shell detection to correctly identify Zsh vs Bash based on user login.
- **Fixed:** Aligned `install.sh` data paths with Rust's native `directories` crate on macOS.

### v0.3.0 (The Architecture Update)
- **Breaking:** Complete engine rewrite to 4-tier hierarchy (Quest -> Chapter -> Task).
- **Breaking:** Renamed `Checkpoint` to `Task` in internal logic.
- **Breaking:** Previous `save.json` files are incompatible. Run with `--reset`.
- **New:** Added support for multi-chapter "Quests" (Seasons).
- **New:** Cinematic Intros and Outros now trigger on Chapter transitions.
- **New:** Added native `Condition` types (e.g., `IsDirectory`, `WorkingDir`) for robust validation.
- **Fixed:** Input handling now correctly accepts `[ENTER]` to advance cutscenes.

### v0.2.0 (The Hierarchy Update)
- **Breaking:** New YAML structure (Chapters & Checkpoints).
- **New:** Automated "Next Objective" reveals after tasks.
- **New:** Added `--reset` flag for easier testing.
- **Fixed:** Prevented "Success Loop" on final missions.

### v0.1.0
- Initial proof of concept with flat quest list.
