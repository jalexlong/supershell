# SuperShell Development Roadmap

## ⚙️ Engine (Rust)
- [ ] **Color Support:** Add `colored` crate or ANSI codes to `ui.rs` for Success/Error themes.
- [ ] **Validation:** Create a `validate` subcommand to check `quests.yaml` for logic errors.
- [ ] **Progress Stats:** Display "% complete" in the `supershell` status command.

## 🐚 Shell Integration
- [ ] **Zsh Support:** Test and document the `precmd` hook for Zsh users.
- [ ] **Auto-Installer:** Create a script to automatically append the hook to `.bashrc`.

## ✅ Done
- [x] Refactor to hierarchical Chapter/Checkpoint structure.
- [x] Implement linear progression with `is_finished` flag.
- [x] Create inherent start-point discovery.
