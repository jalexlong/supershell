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
- `supershell --menu`: Opens the course selector to switch modules.
- `supershell --hint`: Displays a hint for your current task.
- `supershell --validate <file>`: Checks a quest YAML file for errors (useful for content creators).
- `supershell --reset`: Wipes all progress and restarts the engine.

## 📂 Data Locations
Supershell stores game data in your operating system's standard application support directory.

### Linux
* **Content:** `~/.local/share/supershell/library/`
* **Save Data:** `~/.local/share/supershell/save.json`

### MacOS
* **Content:** `~/Library/Application Support/com.jalexlong.supershell/library/`
* **Save Data:** `~/Library/Application Support/com.jalexlong.supershell/save.json`

> **Note for Mac Users:** You can quickly access this folder by opening Finder, pressing `Cmd+Shift+G`, and pasting the path above.
## 🛠️ Installation

### Option 1: The Rust Way (Recommended)
If you have Rust installed, you can grab the game directly from crates.io:

```bash
cargo install supershell
supershell
```

### Option 2: Standalone Binary
1. Download the latest release for your OS from the [Releases Page](https://github.com/jalexlong/supershell/releases).
2. Extract the archive.
3. Run the binary:
   ```bash
   ./supershell
   ```
*(Note: The game will automatically install its game files to your system on the first run.)*

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
            objective: "chmod +r secret.data"
            conditions:
              - type: CommandMatches
                pattern: "^chmod \\+r"

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

## 📦 Changelog
View the full history of changes and updates in [CHANGELOG.md](CHANGELOG.md).

Latest: **v0.5.0 (The Infrastructure Update)** - Added auto-updates, quest validation tools, and metadata support.

## ⚖️ License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
