# Supershell Project Roadmap

## 🛑 Priority 1: Core Mechanics & Safety
- [ ] **Error Handling:** Refactor `src/main.rs` and `src/quest.rs` to replace `.unwrap()` with proper `Result` handling (prevent panics on corrupt save files).
- [ ] **Condition Logic:** Implement the missing logic for `FileExists`, `FileContains`, and `FileMissing` in `quest.rs` (currently only `CommandMatches` is wired up).
- [ ] **Signal Handling:** Add a `Ctrl+C` handler to ensure the terminal state is restored if the user aborts a cutscene (partially handled by `Drop`, but needs verification).

## 🛠️ Priority 2: Distribution & Install
- [ ] **Release Profile:** Optimize `Cargo.toml` for binary size and speed.
- [ ] **Install Script:** Create `install.sh` that:
    - Builds in `--release` mode.
    - Moves binary to `~/.local/bin/`.
    - Creates `~/.local/share/supershell/`.
    - Copies `quests.yaml` to the data folder.
- [ ] **Shell Agnosticism:** Port `supershell.sh` logic to Zsh (`supershell.zsh`).

## 🎨 Priority 3: Polish & Content
- [ ] **Markdown Parsing:** Update the YAML loader to support basic formatting (e.g., `*bold*` turns white in the TUI).
- [ ] **Sound:** (Optional) Add a simple beep or click sound effect during the typewriter animation.
- [ ] **Content Expansion:** Write Chapter 04 (File Creation) and Chapter 05 (Deletion).

## 🐛 Known Issues / Tech Debt
- [ ] `supershell.sh` currently hardcodes the path to `target/debug`. Needs to be dynamic based on install location.
- [ ] The TUI typewriter speed is hardcoded to 25ms. Should be configurable in `config.yaml`.
