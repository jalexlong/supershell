# SuperShell Development Roadmap

## ⚙️ Engine (Rust)
- [ ] **Color Support:** Add `colored` crate or ANSI codes to `ui.rs` for Success/Error themes.
- [ ] **Progress Stats:** Display "% complete" in the `supershell` status command.

## ✅ Done
- [x] Create a `validate` subcommand to check `quests.yaml` for logic errors.
- [x] Create a script to automatically append the hook to `.bashrc`.
- [x] Test and document the `precmd` hook for Zsh users.
- [x] Refactor to hierarchical Chapter/Checkpoint structure.
- [x] Implement linear progression with `is_finished` flag.
- [x] Create inherent start-point discovery.
