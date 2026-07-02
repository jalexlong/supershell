# Changelog

## 📜 Release History

## v0.7.1 — Housekeeping

- Removed unused `_data_dir` field from `AppContext` in `paths.rs`
- Rewrote `TODO.md` to reflect completed milestones and new backlog (M8–M10)
- Updated `docs/playtesting.md` with permissions module walkthrough, failure-path tests, and world-destruction recovery steps

## v0.7.0 — GDD Alignment

- **Glitch effect:** logic failure messages now render with Unicode combining strikethrough (U+0336) on every character, producing the text-corruption visual described in the GDD; suppressed in test mode to keep assertions readable (`ui::glitch_text`)
- **World-destruction recovery:** `WorldEngine::is_intact()` checked at the start of every `--check` call; if `~/Construct` is missing the engine auto-restores the directory and re-runs the current chapter's setup actions
- **`main.rs` → `app.rs` split:** gameplay handlers moved to `src/app.rs`; `main.rs` is now arg-parsing and dispatch only
- Updated `design/GDD.md` to accurately describe alias-based interception and mark Glitch, Hint, and World Reset as implemented
- Updated `design/architecture.md` Section 6 (all stabilization goals Done) and Section 8 divergences table

## v0.6.1 — Hint System

- Added `failure_count: usize` to `GameState` with `#[serde(default)]` for backward compatibility
- `failure_count` increments on logic failure, resets to 0 on success, persisted to disk
- After 3 consecutive failures, `task.hint` shown below the failure card in yellow
- Added `hint:` fields to all tasks in `library/intro.yaml` and `tests/fixtures/mock_quest.yaml`
- New integration tests: `hint_shown_after_three_failures`, `hint_resets_on_task_success`

## v0.6.0 — HistoryContains + Access Control Module

- **`HistoryContains` implemented:** reads `$HISTFILE` (falls back to `~/.bash_history`); regex match; warns on invalid pattern
- **`history -a` flush** in `_g` alias so current command is visible to the next check
- **`export HISTFILE`** added to RC template; `alias chmod='_g chmod'` added to infection hooks
- New module `library/permissions.yaml` — teaches `ls -la`, `chmod +x`, and octal permissions; exercises `IsExecutable`, `FileContains`, `FlagIsTrue`, and flag reward chains
- New integration tests: `history_contains_passes_when_pattern_found`, `history_contains_fails_when_pattern_absent`

## v0.5.3 — Interactive Menu

- `ui::show_module_menu()` replaces the `show_menu` stub; arrow-key nav via crossterm, numbered fallback when raw mode unavailable
- Auto-selects when only one module exists or in `SUPERSHELL_TEST_MODE`
- `--menu` returns immediately after saving selection; no longer falls through to shell launch
- Inside-Construct message correctly tells the user to `exit` and rerun
- New integration tests: `menu_auto_selects_single_module`, `menu_selection_persists_for_status`

## v0.5.2 — Test Coverage

- New test fixture `tests/fixtures/mock_quest.yaml` — two-chapter quest covering `CommandMatches`, `WorkingDir`, `PathExists`, `FlagIsTrue`, `SetFlag` reward, and hint text
- Six new integration tests: `state_persists_across_invocations`, `failure_returns_exit_code_1`, `refresh_succeeds_after_reset`, `multi_task_progression`, `chapter_transition_triggers_setup`, `reward_application_enables_gated_task`
- Updated `CONTRIBUTING.md` with mock quest fixture docs and two-invocation test pattern

## v0.5.1 — Panic Hardening

- All `expect`/`unwrap`/`panic!` calls in I/O paths replaced with `anyhow::Result` or graceful degradation
- `Course::load()` returns `anyhow::Result<Course>` instead of panicking on bad YAML
- Invalid regex warns to stderr and fails closed
- `ui.rs`: raw mode failures fall back to `render_plain_card`; `disable_raw_mode` uses `.ok()`
- `shell.rs`: `launch_infected_session()` returns `anyhow::Result`; all `.expect()` calls removed
- `world.rs`: `WorldEngine::new()` and `initialize()` return `anyhow::Result`
- `state.rs`: parse errors emit a `stderr` warning instead of silently returning fresh state

## v0.5.0 — The Transient Shell Update

**Architecture:**
Replaced persistent shell hook installation with a transient bash session launched directly by the `supershell` binary. The session uses a generated temporary RC file containing a `_g` alias interceptor. The user's normal shell is untouched; the Construct is a separate sandboxed environment the user enters and exits voluntarily.

**Security:**
Sandbox path safety added to both `WorldEngine` (setup actions) and condition evaluation (all filesystem conditions). All YAML-specified paths are constrained to `~/Construct` via `resolve_construct_path`, which rejects `..` traversal, absolute paths, Windows-style prefixes, and empty input.

**Engine:**
Two-pass validation engine: a relevance pass (`CommandMatches` conditions) determines whether the command applies to the current task without any visible feedback; a logic pass (all remaining conditions) runs only if the relevance pass succeeds and blocks the task with a failure message if anything fails. Exit code contract: 0 = irrelevant, 1 = logic failure, 2 = task complete (bash should refresh UI).

**Persistence:**
Atomic save via `.tmp` write then POSIX rename. Test isolation via `SUPERSHELL_TEST_MODE=1` + `XDG_DATA_HOME` env vars so automated tests never touch real user state.

**DX:**
- `--validate <path>` — check YAML syntax and print course metadata without launching a session
- `CONSTRUCT_UPLINK=1` guard — prevents nested Construct sessions
- Constrained-terminal fallback — narrow or short terminal panes fall back to a plain-text renderer instead of the animated box UI

**Content:**
Introductory module rewritten to canonical 4-chapter structure: System Boot (`ls`), Motor Functions (`cd`), The Labyrinth (navigation), Data Processing (`cat`). All chapters use YAML-driven `setup_actions` to build the scenario in `~/Construct`.

---

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
