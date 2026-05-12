# Supershell

**Supershell** is a terminal-based learning RPG that teaches command-line skills through real shell interaction, quests, objectives, rewards, and narrative feedback.

The current v0.5 architecture launches a transient guided shell session instead of relying on legacy shell startup hooks. The goal is to preserve the user's real shell behavior while letting Supershell observe relevant commands, validate objectives, and refresh the mission display when progress changes.

## Current Status

Supershell is in active development.

The current stabilization target is:

- reliable local development workflow
- deterministic quest loading
- safe save-state handling
- YAML-driven lesson content
- transient shell session support
- regression tests for reset, status, and command-check behavior

## How to Run

From the repository root:

```bash
cargo run
```

This launches the guided Supershell session.

To view the current mission status directly:

```bash
cargo run -- --status
```

To open the module selector:

```bash
cargo run -- --menu
```

To reset progress:

```bash
cargo run -- --reset --status
```

To validate a quest YAML file:

```bash
cargo run -- --validate library/intro.yaml
```

To simulate a command check during development:

```bash
cargo run -- --check "ls"
```

## Development Commands

Run the standard local quality gate before committing:

```bash
cargo fmt --check
cargo check
cargo test
```

During active editing, you can allow formatting to update files:

```bash
cargo fmt
cargo check
cargo test
```

## Core Concepts

Supershell is built around a small loop:

```text
User command -> Supershell check -> Quest conditions -> State update -> UI refresh
```

The major pieces are:

- `src/main.rs` — CLI entry point and application flow
- `src/shell.rs` — transient shell session support
- `src/quest.rs` — YAML quest model and condition validation
- `src/state.rs` — persistent save data
- `src/ui.rs` — terminal rendering and feedback
- `src/world.rs` — Construct setup and scenario-building
- `library/intro.yaml` — bundled introductory quest content

## Quest Content

Quest content is YAML-driven. Lessons are organized as:

```text
Course -> Quests -> Chapters -> Tasks
```

Tasks use conditions to determine whether the user has completed an objective.

Example:

```yaml
tasks:
  - objective: "List the files in the current directory."
    instruction: "Use ls to scan the room."
    success_msg: "Sensors Online."
    conditions:
      - type: CommandMatches
        pattern: "^ls(\\s.*)?$"
```

## Save Data

Supershell stores progress in the operating system's standard application data directory.

During tests, Supershell uses an isolated test data directory so test runs do not touch real user save data.

## The Construct

Many missions take place inside a generated sandbox directory called **The Construct**:

```text
~/Construct
```

The world engine can create files, folders, and puzzle scenarios inside this directory based on YAML setup actions.

## Playtesting

Manual playtesting instructions are available in [`docs/playtesting.md`](docs/playtesting.md).

Run the playtest after changing quest content, shell behavior, UI rendering, or progression logic.

## Project Direction

The long-term goal is a stable, classroom-ready cybersecurity education tool that:

- teaches real command-line skills
- keeps lesson content data-driven
- avoids breaking the user's normal shell
- fails safely when something goes wrong
- supports narrative-driven learning
- remains maintainable for educators and contributors

## License

This project is licensed under the MIT License. See `LICENSE` for details.
