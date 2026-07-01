# TODO — Supershell Development Backlog

Items are grouped by urgency. File paths include line numbers where the issue lives in the current source.

---

## Hot — Panics / User-Visible Crashes (M1)

These can crash the binary or leave the terminal in a broken state. Fix before shipping to any new users.

- [ ] `src/quest.rs:94` — `panic!("YAML Error: {}", e)` — convert `Course::load()` to return `Result<Course, anyhow::Error>`; skip unparseable files in `list_available_courses()`
- [ ] `src/quest.rs:224,233` — invalid regex silently falls back to `Regex::new("").unwrap()` (never matches); emit `stderr` warning and return `SyntaxError` instead
- [ ] `src/ui.rs:259` — `disable_raw_mode().unwrap()` can leave the terminal in raw mode on panic; change to `disable_raw_mode().ok()`
- [ ] `src/ui.rs:132` — `enable_raw_mode()` result not handled; wrap in error check and fall back to `render_plain_card` on failure
- [ ] `src/shell.rs:96,101,106,118,120,133` — six `.expect()` calls crash the binary if home dir is missing or exe path can't be resolved; convert `launch_infected_session()` to `Result<(), anyhow::Error>`
- [ ] `src/world.rs:13` — `WorldEngine::new()` calls `.expect("Critical: Could not find User Home.")` — convert to `Result<Self, anyhow::Error>`
- [ ] `src/world.rs:22` — `initialize()` calls `.expect("Failed to create world root.")` — propagate as `Result`
- [ ] `src/main.rs:99,123,133` — three `.expect("Failed to save game state")` calls; route all saves through the existing `save_game_state()` helper
- [ ] `src/state.rs::load()` — silently returns fresh state on JSON parse error; emit `stderr` warning and rename corrupted file to `{path}.bak` before returning `Self::new()`

---

## Stubs — Incomplete Implementations

Features that compile and appear functional but do nothing or do the wrong thing.

- [ ] `src/quest.rs:231–235` — `HistoryContains` always matches `"TODO_IMPLEMENT_HISTORY_READ"`; add `eprintln!` warning so quest authors know this condition is not evaluated (M1 minimal fix); full implementation is M4
- [ ] `src/main.rs:194–208` — `show_menu()` always selects the first course; prints "Selecting module 1 by default (Menu UI WIP)"; real interactive selector is M3
- [ ] `src/main.rs:149–157` — `--menu` falls through to `launch_infected_session()` after saving the selection; add `return Ok(())` after the save so selecting a menu item doesn't immediately launch the shell
- [ ] `src/paths.rs` — `AppContext._data_dir` field is constructed but never read (the `_` prefix suppresses the lint); either use it or remove it

---

## Test Gaps (M2)

`tests/cli_workflow.rs` has 5 tests. The following cases have no coverage:

- [ ] Create `tests/fixtures/mock_quest.yaml` — two-chapter quest covering: `CommandMatches`, `WorkingDir`, `PathExists`, `FlagIsTrue`, `SetFlag` reward, hint text on each task
- [ ] `state_persists_across_invocations` — complete a task in one process, assert the state update is visible in a second process invocation
- [ ] `failure_returns_exit_code_1` — run `--check` with a command that matches the regex but fails a logic condition (e.g., `FlagIsTrue` not set); assert exit code 1 and `[FAIL]` in stdout
- [ ] `refresh_succeeds_after_reset` — `--reset` then `--refresh`; assert no panic and stdout contains expected status output
- [ ] `multi_task_progression` — complete tasks 1 and 2 of mock quest sequentially; assert `current_task_index` advances correctly
- [ ] `chapter_transition_triggers_setup` — completing the final task of chapter 1 should cause chapter 2's `setup_actions` to run; assert the expected `PathExists` condition passes in a subsequent check
- [ ] `reward_application_updates_state` — complete a task with a `SetFlag` reward; assert that a subsequent `--check` requiring `FlagIsTrue` passes

---

## Feature Backlog

### M3 — Interactive Menu (v0.5.3)
- [ ] Move `show_menu()` from `main.rs` to `ui.rs`
- [ ] Implement arrow-key navigation using crossterm (fallback to numbered input when raw mode unavailable)
- [ ] Auto-select without displaying a menu when only one module exists
- [ ] Improve the "already inside Construct" message for `--menu` called from a live session: explain that the user must `exit` and relaunch to apply the selection

### M4 — HistoryContains + Second Module (v0.6.0)
- [ ] Add `history -a` to the `_g` function in `SHELL_RC_TEMPLATE` (`src/shell.rs`) to flush in-session history before the `--check` call
- [ ] Implement `HistoryContains::check()` in `src/quest.rs` by reading `$HISTFILE` (fall back to `~/.bash_history`)
- [ ] Create `library/permissions.yaml` — module teaching `chmod`, `ls -la`, and octal permission reading; exercises `IsExecutable`, `FileContains`, and `FlagIsTrue` reward chains

### M5 — Hint System (v0.6.1)
- [ ] Add `failure_count: usize` to `GameState` (`src/state.rs`); reset to 0 in `advance_task()`
- [ ] Increment `failure_count` in `handle_check_command` on `LogicFailure` and persist
- [ ] Pass `task.hint` through to `print_fail()` in `src/ui.rs`; show hint after 3 failures on same task
- [ ] Populate `hint` fields in `library/intro.yaml` for all 5 tasks

### M6 — GDD Alignment (v0.7.0)
- [ ] Implement `print_glitch()` in `src/ui.rs` — failure message with Unicode strikethrough (U+0336) and inverted red/white color; call from `handle_check_command` on `LogicFailure`
- [ ] World-destruction detection in `handle_check_command` — if `~/Construct` is missing before command evaluation, call `world.build_scenario()` to restore before proceeding
- [ ] `src/main.rs` orchestration split — move `handle_check_command`, `handle_status_display`, `handle_refresh_sequence`, `perform_validation`, and `CheckCommandOutcome` to `src/app.rs`; `main.rs` becomes arg-parsing and dispatch only
- [ ] Update `design/GDD.md` — replace "implicit hooking" claim with accurate description of alias-based `_g` interceptor; mark Glitch/hints/world-reset as "planned"

---

## Documentation Debt

- [ ] `CHANGELOG.md` — add v0.5.0 entry (transient shell architecture, sandbox path safety, two-pass engine, atomic save, test isolation, `--validate`, `CONSTRUCT_UPLINK`, constrained-terminal fallback)
- [ ] `design/architecture.md` Section 6 — annotate each stabilization goal with Done / In Progress / Not Started; add Section 8 "Known Divergences from GDD"
- [ ] `CONTRIBUTING.md` — add YAML schema reference (currently in `CLAUDE.md`); mention `tests/fixtures/mock_quest.yaml` and the two-invocation state-persistence test pattern
- [ ] `design/GDD.md` — reconcile with current implementation (see M6)
- [ ] `docs/playtesting.md` — add failure-path test cases (wrong command, wrong directory, repeated failure triggering hint); document what each prompt and transition looks like for regression checking
