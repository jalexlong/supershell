# Supershell Architecture

## 1. Overview

Supershell is a gamified command-line learning system. It teaches shell concepts through quests, objectives, narrative feedback, and stateful progression.

The current v0.5 implementation uses a **transient shell architecture**. Instead of permanently modifying the user's shell startup files through legacy hooks, Supershell launches a guided shell session and evaluates relevant commands through its Rust runtime.

The long-term design goal remains:

```text
real shell behavior first, game overlay second
```

Supershell should enhance the terminal experience without making the user's normal shell fragile.

## 2. Runtime Flow

The simplified runtime loop is:

```text
Launch Supershell
  -> initialize paths
  -> extract bundled quest library
  -> load save state
  -> resolve active course
  -> initialize Construct world
  -> launch guided shell or handle CLI command
```

When checking a command:

```text
command string
  -> active task
  -> relevance conditions
  -> strict validation conditions
  -> rewards
  -> state transition
  -> save state
  -> UI refresh signal
```

## 3. Major Components

### 3.1 CLI Entry Point and Application Logic

Files:

```text
src/main.rs   — arg parsing, path setup, dispatch
src/app.rs    — gameplay handlers (handle_check_command, handle_status_display, handle_refresh_sequence, perform_validation)
```

`main.rs` responsibilities:

- parse CLI arguments
- construct application paths
- extract bundled quest content
- load or reset save state
- resolve the active course
- dispatch to status, menu, refresh, validation, check, or shell-launch behavior

`app.rs` responsibilities:

- two-pass command evaluation (relevance → logic)
- failure-count tracking and hint surfacing
- world-destruction detection and auto-restore
- progression state transitions
- save/reset helpers

Current CLI flags include:

```text
--check
--reset
--validate
--menu
--status
--refresh
```

### 3.2 Transient Shell

File:

```text
src/shell.rs
```

Responsibilities:

- launch a guided shell session
- provide the interactive environment where Supershell can observe user commands
- preserve the user's ability to run normal shell commands

This replaces the older design that depended on persistent shell startup hooks.

### 3.3 Quest Engine

File:

```text
src/quest.rs
```

Responsibilities:

- define the course, quest, chapter, task, condition, and reward data models
- load YAML quest content
- validate command and state conditions
- support stateful rewards such as flags and variables

The current engine uses a two-pass validation model:

1. **Relevance pass** — determine whether the command applies to the current task.
2. **Logic pass** — verify additional requirements such as state, filesystem, or world conditions.

### 3.4 State Manager

File:

```text
src/state.rs
```

Responsibilities:

- load save data from JSON
- create a default state when no save exists
- persist progress
- preserve flags and variables
- use atomic write behavior through temporary-file write followed by rename

The save layer must ensure its parent directory exists before writing.

### 3.5 UI Renderer

File:

```text
src/ui.rs
```

Responsibilities:

- render mission status
- render success and failure messages
- display cutscene text
- support test-mode behavior for non-interactive test runs

The UI should avoid making the shell unusable if rendering fails.

### 3.6 World Engine

File:

```text
src/world.rs
```

Responsibilities:

- initialize the Construct
- create scenario files and folders
- apply YAML-defined setup actions
- keep generated mission content inside the intended sandbox

The Construct currently lives at:

```text
~/Construct
```

### 3.7 Bundled Quest Library

Directory:

```text
library/
```

Responsibilities:

- provide built-in quest YAML files
- serve as the default lesson library
- get embedded into the Rust binary and extracted at runtime

Current bundled modules:

```text
library/tutorial.yaml     — narrative cold open; ls and cd
library/intro.yaml        — guided orientation arc; ls, cd, cat
library/permissions.yaml  — access control arc; ls -la, chmod
```

## 4. Persistence Model

Supershell stores save data in the operating system's application data directory.

Normal runtime:

```text
<platform data dir>/supershell/save.json
```

Test runtime:

```text
$XDG_DATA_HOME/supershell/save.json
```

The test-mode path is intentionally isolated so automated tests do not touch real user state.

## 5. Design Priorities

The project should optimize for:

1. **Safety** — Supershell must not break the user's normal terminal.
2. **Clarity** — the engine should stay understandable to educators and contributors.
3. **Data-driven content** — lesson changes should usually happen in YAML, not Rust.
4. **Testability** — command checking, state transitions, and persistence should be covered by regression tests.
5. **Classroom reliability** — failures should be recoverable, explainable, and non-destructive.

## 6. Near-Term Stabilization Goals

Before adding more features, the project should focus on:

- expanding CLI workflow tests — **Done** (39 tests: 22 unit + 17 integration; coverage added M2–M5)
- making state persistence return `Result` instead of panicking — **Done** (M1: all `expect`/`unwrap` in I/O paths removed; `anyhow::Result` throughout)
- separating command evaluation from UI rendering — **Done** (`engine.rs` is pure logic; `ui.rs` is rendering; no cross-dependency)
- reducing `main.rs` orchestration complexity — **Done** (M6: gameplay handlers moved to `src/app.rs`; `main.rs` is arg-parsing + dispatch only)
- documenting the YAML schema — **Done** (schema reference in `CLAUDE.md`)
- validating Construct path safety — **Done** (`construct.rs` fully implemented and tested)
- preserving fail-open behavior in shell-adjacent paths — **Done** (M1: `ui.rs` constrained-terminal fallback; raw-mode errors handled with `.ok()`)

## 7. Future Architecture Direction

A future simplified engine should move toward this shape:

```text
Event + GameState + Course -> Actions + Updated GameState
```

That would allow Supershell to keep a flexible content system while making the Rust core smaller and easier to test.

## 8. Known Divergences from GDD

The GDD (`design/GDD.md`) was written before the transient shell architecture and describes several things that differ from the current implementation:

| GDD claim | Current reality | Status |
|---|---|---|
| "The engine hooks the shell command implicitly" | Alias-based `_g` interceptor in the temp RC file; `--check` is an implementation detail invisible to the player | Resolved — UX is identical to intent |
| "Glitch" visual effect (text corruption on failure) | `glitch_text()` in `ui.rs` applies U+0336 combining strikethrough to the failure message; suppressed in test mode | Done — M6 |
| Hint injection on repeated failure ("Corrupted Data Fragment") | `failure_count` in `GameState`; hint shown after 3 consecutive failures; resets on success | Done — M5 |
| "World Reset on destruction" (auto-restore if `~/Construct` deleted) | `world.is_intact()` checked at the start of every `--check` call; auto-restores via chapter setup actions | Done — M6 |
