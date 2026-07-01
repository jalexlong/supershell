# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Supershell is a terminal-based RPG that teaches command-line and cybersecurity skills. It launches a transient `bash` session (the "Construct") with hooked aliases that intercept commands and pipe them to the Rust binary for quest validation.

## Commands

### Quality gate (run before committing)
```bash
cargo fmt --check
cargo check
cargo test
```

### Running the app
```bash
cargo run                          # launch the interactive shell session
cargo run -- --status              # show current mission card
cargo run -- --menu                # open module selector
cargo run -- --reset --status      # wipe save data and show fresh status
cargo run -- --validate library/intro.yaml   # validate a YAML module file
cargo run -- --check "ls"          # simulate a command check (dev only)
```

### Running a single test
```bash
cargo test test_name               # by name substring
cargo test --test cli_workflow     # run only the integration test file
```

## Architecture

### Runtime loop
```
cargo run → main.rs → shell::launch_infected_session()
  → spawns bash with a temp RC file containing hooked aliases
  → user types "ls" → alias calls `_g ls`
  → _g runs ls, then calls: supershell --check "ls" --cwd "$PWD"
  → Rust binary: relevance check → logic check → state update → save → exit 2
  → bash sees exit code 2 → calls: supershell --refresh (clears screen, shows new task)
```

### Key source files

| File | Role |
|------|------|
| `src/main.rs` | CLI parsing, application orchestration, `--check` handler |
| `src/shell.rs` | Builds and launches the bash RC file; sets `CONSTRUCT_UPLINK` to prevent nesting |
| `src/engine.rs` | Two-pass command validation: relevance (CommandMatches) then logic (all other conditions) |
| `src/quest.rs` | YAML data model (`Course → Quest → Chapter → Task → Condition`) and condition evaluation |
| `src/state.rs` | Save/load `GameState` as JSON; atomic write via `.tmp` + rename |
| `src/paths.rs` | Resolves data dir; switches to `$XDG_DATA_HOME/supershell` when `SUPERSHELL_TEST_MODE=1` |
| `src/world.rs` | Executes `SetupAction`s to build/tear down the `~/Construct` sandbox |
| `src/actions.rs` | `SetupAction` enum: `CreateDir`, `CreateFile`, `RemovePath`, `ResetWorld` |
| `src/ui.rs` | Terminal rendering (status card, success/fail messages, cutscenes) |
| `src/construct.rs` | Sandbox path safety — rejects `..` traversal and absolute paths |
| `library/intro.yaml` | Bundled into the binary via `include_dir!`; extracted to data dir on every run |

### Exit code contract (`--check`)
- `0` — command irrelevant to current task (silent pass-through)
- `1` — logic failure (wrong context; Rust prints the failure message)
- `2` — task completed; bash should clear and call `--refresh`

### Test isolation
Integration tests in `tests/cli_workflow.rs` set `SUPERSHELL_TEST_MODE=1` and `XDG_DATA_HOME=<TempDir>` so they never touch real user save data. Always set both env vars when writing new integration tests.

## Quest YAML Schema

```
Course
  title, author, version
  quests[]
    id, title, construct (bool)
    chapters[]
      title, intro, outro
      setup_actions[]   # SetupAction variants
      tasks[]
        objective, description, instruction, success_msg, hint
        conditions[]    # ConditionType variants (tag = "type")
        rewards[]       # Reward variants (tag = "type")
```

### Condition types
**Relevance (pass 1):** `CommandMatches`, `HistoryContains`
**Logic (pass 2):** `PathExists`, `PathMissing`, `IsDirectory`, `IsFile`, `IsExecutable`, `FileContains`, `FileNotContains`, `FileEmpty`, `WorkingDir`, `EnvVar`, `FlagIsTrue`, `VarEquals`, `VarGreaterThan`, `VarLessThan`

All filesystem conditions in quest YAML are sandboxed to `~/Construct`. Absolute paths and `..` traversal silently return `None` (fail-closed).

### Reward types
`SetFlag { key, value }`, `SetVar { key, value }`, `AddVar { key, amount }`

## Known Stubs

These exist in the codebase and need attention before being used in quest content:

- **`HistoryContains` condition** (`src/quest.rs:231`) — always returns `false`; the history-read is not implemented. Emits a `stderr` warning after M1. Do not use in quest YAML until M4.
- **`show_menu()`** (`src/main.rs:194`) — always selects the first course and prints "Selecting module 1 by default (Menu UI WIP)". Real arrow-key selection is M3.
- **`_data_dir` field** on `AppContext` (`src/paths.rs`) — constructed but never read; the underscore suppresses the unused-field warning. Either wire it up or delete it.

## Error Handling Conventions

- Functions that perform I/O or access system resources return `anyhow::Result<T>`. The `anyhow` crate is already in `Cargo.toml` and `run()` in `main.rs` already uses it.
- UI functions (`ui.rs`) degrade gracefully on terminal errors — they do not propagate errors to callers. A failed `enable_raw_mode` falls back to `render_plain_card`.
- `.unwrap()` is permitted only in test code and in compile-time / `const` contexts.
- Current violations (all targeted in M1): `quest.rs:94` panics on bad YAML; `ui.rs:259` `disable_raw_mode().unwrap()` can corrupt the terminal; `shell.rs` has six `.expect()` calls; `world.rs:13,22` panics on missing home dir.

## `CONSTRUCT_UPLINK` Behavior

The transient shell session (`shell.rs`) sets the env var `CONSTRUCT_UPLINK=1` before spawning bash. This prevents nested Construct sessions: if the user runs `supershell` from inside the Construct, the binary detects the var and prints a warning instead of launching another shell.

When `--menu` is called from inside a live session, the selection is saved to disk but the binary cannot relaunch the shell (the current session is already running). The message should tell the user to `exit` and relaunch. Do not remove the `CONSTRUCT_UPLINK` check — it is load-bearing.

## Integration Test Pattern

All integration tests must set two env vars to avoid touching real user save data:

```rust
fn test_env<'a>(cmd: &'a mut assert_cmd::Command, temp: &TempDir) -> &'a mut assert_cmd::Command {
    cmd.env("SUPERSHELL_TEST_MODE", "1")
       .env("XDG_DATA_HOME", temp.path())
}
```

For tests that verify **state persistence across invocations**, use two separate `Command` calls against the same `TempDir`:

```rust
let temp = TempDir::new().unwrap();
// First invocation — complete a task
test_env(&mut supershell(), &temp).arg("--check").arg("ls").assert().code(2);
// Second invocation — verify the state update persisted
test_env(&mut supershell(), &temp).arg("--status").assert().success();
```

The mock quest fixture at `tests/fixtures/mock_quest.yaml` (added in M2) covers `CommandMatches`, `WorkingDir`, `PathExists`, `FlagIsTrue`, `SetFlag` reward, and hint text — use it instead of the real `intro.yaml` when testing engine logic.

## Milestone Roadmap

| Milestone | Target | Theme |
|---|---|---|
| M1 | v0.5.1 | Panic hardening — remove all crash-or-corrupt `unwrap`/`expect`/`panic!` |
| M2 | v0.5.2 | Test coverage — mock quest fixture + 6 new integration tests |
| M3 | v0.5.3 | Interactive menu — arrow-key selection, single-module auto-select |
| M4 | v0.6.0 | `HistoryContains` implementation + `library/permissions.yaml` module |
| M5 | v0.6.1 | Hint system — `failure_count` in state, `task.hint` surfaced after 3 fails |
| M6 | v0.7.0 | GDD alignment — Glitch effect, world-destruction detection, GDD reconciliation |

See `TODO.md` for the full backlog.

## Design Priorities

1. **Safety first** — never break the user's normal shell; fail-open on render errors
2. **Data-driven content** — lesson changes belong in YAML, not Rust
3. **Testability** — command evaluation and state transitions should have regression coverage
4. **Classroom reliability** — failures must be recoverable and non-destructive
