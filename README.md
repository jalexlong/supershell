# SuperShell Tutor

A data-driven terminal curriculum engine built in Rust. It transforms the standard shell into an interactive learning environment using a background hook system.

## 🚀 How it Works
SuperShell monitors your terminal activity via a shell hook. When you complete a task defined in `quests.yaml`, the engine provides immediate feedback, plays narrative cutscenes, and advances the quest state.

## 🛠 Features
- **Hierarchical Design:** Lessons are organized into Quests (Seasons), Chapters (Episodes), and atomic Tasks.
- **Narrative-Driven:** Separate fields for "Flavor Text" and "Technical Objectives."
- **Persistent State:** Progress is saved automatically to your system's standard data directory.
- **Hybrid Validation:** Uses a mix of system checks (`IsDirectory`, `PathMissing`) and Regex pattern matching to verify objectives.

## ⌨️ Commands
- `supershell`: Displays the current chapter title and your active objective.
- `supershell --reset`: Wipes all progress and restarts the curriculum from the beginning.

## 📂 Data Locations (Linux/MacOS)
Supershell adheres to the XDG Base Directory Specification:
* **Content:** `~/.local/share/supershell/quests.yaml`
* **Save Data:** `~/.local/share/supershell/save.json`

## ⚖️ License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📜 Release History

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
