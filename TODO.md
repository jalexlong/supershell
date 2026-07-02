# TODO — Supershell Development Backlog

Items are grouped by urgency. All M1–M6 milestones are complete as of v0.7.0.

---

## Known Stubs

Compile and appear functional but have incomplete or edge-case behavior.

- [ ] `src/quest.rs` — `HistoryContains` reads `$HISTFILE` correctly, but the `_g` alias flushes history with `history -a` *after* the command runs. A command is not visible to the check until the *next* command. Design quest tasks that use `HistoryContains` to require a follow-up command (e.g., the pattern is checked on the next relevant command, not the one being matched). Document in quest YAML schema notes.
- [ ] `src/main.rs` — `--menu` from inside a live Construct session saves the selection but cannot relaunch the shell. The message correctly tells the user to `exit` and rerun. UX is functional but awkward; revisit if a persistent daemon model (M10) is adopted.

---

## Feature Backlog

### M8 — Third Module: Navigation (v0.5.8)
A third content module teaching file search and text processing:
- `find` — locate files by name, type, and depth
- `grep` — pattern search within files
- `|` pipe — chain commands together
- Exercises: `IsFile`, `FileContains`, multi-condition tasks, pipe-composed commands

### M9 — Score & Replay (v0.5.9)
- Track per-task completion time and failure count in `GameState`
- `--score` flag: render a summary card after module completion
- Allow replaying completed modules from `--menu` (currently `is_finished = true` blocks replays)

### M10 — Persistent Daemon Mode (v0.6.0)
- Replace the transient shell + alias approach with a background daemon that the shell RC sources on every prompt via `PROMPT_COMMAND`
- Enables: history-flush before check, per-prompt state sync, richer shell instrumentation
- Breaking change to the shell integration model; requires careful migration path

### M11 — Quest Editor GUI (v1.0.0)
A dedicated visual tool that lets educators build quest content without touching YAML directly or reading engine documentation:
- GUI form for each schema level (Course → Quest → Chapter → Task → Condition → Reward)
- Condition type picker with field descriptions and examples inline
- Live preview of the generated YAML
- One-click validation (calls `supershell --validate` under the hood)
- Export to a `.yaml` file ready to drop into the `library/` directory
- Possible implementation: a web-based tool (Tauri app or simple localhost server) so it works cross-platform without a native GUI dependency

---

## Documentation Debt

- [ ] Quest YAML schema — document the one-command `HistoryContains` lag so module authors know to design around it
- [ ] `docs/playtesting.md` — add permissions module walkthrough and M4–M6 failure-path test cases
