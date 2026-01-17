# SuperShell 🐚 🚀

**SuperShell** is a terminal-based RPG that turns your command line into a video game. 

It runs silently in the background of your actual shell (Zsh/Bash). As you navigate your file system and run real commands, SuperShell tracks your progress, unlocks chapters, and guides you through a sci-fi narrative—all while teaching you actual CLI skills.

## 🎮 How to Play

### 1. The Game
Start the game by launching the first mission:
```bash
supershell
```
### 2. The Menu
Or start the game by opening the mission selector:
```bash
supershell --menu
```

### 3. Hints
If you're stuck on a certain task, try to get a hint from the system.
```bash
supershell --hint
```

## 🚀 How it Works
SuperShell monitors your terminal activity via a shell hook. When you complete a task defined in a quest's YAML file located in the `library/` directory, the engine provides immediate feedback, plays narrative cutscenes, and advances the quest state.

## 🛠 Features
- **Hierarchical Design:** Lessons are organized into Quests (Seasons), Chapters (Episodes), and atomic Tasks.
- **Narrative-Driven:** Separate fields for "Flavor Text" and "Technical Objectives."
- **Persistent State:** Progress is saved automatically to your system's standard data directory.
- **Hybrid Validation:** Uses a mix of system checks (`IsDirectory`, `PathMissing`) and Regex pattern matching to verify objectives.

## ⌨️ Commands
- `supershell`: Displays the current chapter title and your active objective.
- `supershell --reset`: Wipes all progress and restarts the curriculum from the beginning.
- `supershell --menu`: Displays the quest/module selection menu.
- `supershell --hint`: Displays a hint for the current in-game task.

## 📂 Data Locations (Linux/MacOS)
Supershell adheres to the XDG Base Directory Specification:
* **Content:** `~/.local/share/supershell/quests.yaml`
* **Save Data:** `~/.local/share/supershell/save.json`

## ⚖️ License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📜 Release History

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

# SuperShell 🐚 🚀

**SuperShell** is a terminal-based RPG that turns your command line into a video game. 

It runs silently in the background of your actual shell (Zsh/Bash). As you navigate your file system and run real commands, SuperShell tracks your progress, unlocks chapters, and guides you through a sci-fi narrative—all while teaching you actual CLI skills.

## 🎮 How to Play

### 1. The Menu
Start the game by opening the mission selector:
```bash
supershell --menu

```

Select a module (e.g., `01_awakening`). This initializes your save file.

### 2. The Hook (Magic Mode)

Once a mission is active, **just use your terminal normally**.
Every time you run a command (like `cd`, `ls`, or `cat`), SuperShell checks if you completed the current objective.

* **Status Check:** If you forget what to do, just hit `ENTER` on an empty line.
* **Hints:** Stuck? Run `supershell --hint`.
* **Reset:** If you break the game world, run `supershell --reset` to wipe your save and start over.

---

## 🛠️ Installation

### Prerequisites

* **Rust** (Cargo) installed.
* **Zsh** or **Bash** shell.

### Step 1: Build

Clone the repo and build the release binary:

```bash
cargo build --release

```

### Step 2: Install

Run the provided install script. This will:

1. Copy the binary to a local path (e.g., `~/.local/bin`).
2. Copy the quest library to your data folder (`~/.local/share/supershell`).
3. Add the **Shell Hook** to your `.zshrc` or `.bashrc`.

```bash
chmod +x install.sh
./install.sh

```

> **Note on the Shell Hook:**
> The installer adds a small function to your shell config. This function runs `supershell --check "$HISTORY"` after every command you type, allowing the game to react to your actions in real-time.

---

## 📂 The Construct

Most missions take place inside a safe, sandboxed directory called **The Construct**:

```text
~/Construct

```

The game will automatically generate files, folders, and puzzles inside this directory. You can delete it at any time; the game will rebuild it when you load a mission.

## 🧱 Creating Custom Quests

SuperShell is data-driven. You can write your own missions using YAML files in the `library/` folder.

**Example Quest Structure:**

```yaml
quests:
  - id: "03_permissions"
    title: "Module 03: Security"
    construct: true  # Requires user to be in ~/Construct
    chapters:
      - title: "The Locked Door"
        intro: "You encounter a file you cannot read..."
        setup_actions:
          - type: CreateFile
            path: "secret.data"
            content: "TOP SECRET"
        tasks:
          - description: "Change permissions"
            objective: "chmod +x secret.data"
            conditions:
              - type: CommandMatches
                pattern: "^chmod \\+x"

```

---

## 🐛 Troubleshooting

**"The game isn't reacting!"**

1. Ensure you have selected a mission via `supershell --menu`.
2. Restart your terminal to ensure the shell hook is loaded.
3. Check if you are inside `~/Construct` (if the mission requires it).

**"I want to uninstall"**
Remove the `supershell` binary and delete the lines added to your `.zshrc` or `.bashrc`.

```bash
rm -rf ~/.local/share/supershell

```
