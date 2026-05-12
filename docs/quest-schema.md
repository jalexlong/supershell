# Supershell Quest Schema Reference

This document describes the current v0.5 Supershell quest content format.

Supershell content is written in YAML. The preferred canonical structure is:

```text
Course -> quests -> chapters -> tasks
```

A course may contain one or more quests. Each quest contains chapters. Each chapter contains tasks. Each task contains conditions that determine whether the player's real shell command should advance progress.

## Design Rule

Quest content should follow Supershell's core design pillar:

```text
real shell behavior first, game overlay second
```

Content should teach the real shell. The story layer should reinforce shell concepts, not replace them.

For the Derelict Data Ark theme:

```text
Directories are places.
Files are objects, items, or things.
Commands are actions.
```

## Minimal Course Example

```yaml
title: "Module 1: Reactivation"
author: "Supershell Team"
version: "1.0.0"

quests:
  - id: "reactivation"
    title: "Cold Start"
    construct: true

    chapters:
      - title: "Emergency Light"
        intro: |
          Operator, your console is active.
          Use `ls` to scan the current place.

        setup_actions:
          - type: ResetWorld
          - type: CreateDir
            path: "Memory_Bank"

        tasks:
          - objective: "Scan the Training Area"
            description: "Use `ls` to scan your current place."
            instruction: "Run `ls`."
            success_msg: "Scan complete."
            conditions:
              - type: CommandMatches
                pattern: '^ls(\s.*)?$'

        outro: |
          Scan complete. A new place is visible.
```

## Course Fields

Top-level course fields:

| Field | Required | Type | Purpose |
|---|---:|---|---|
| `title` | No | string | Display title for the course/module. Defaults to `Untitled Course`. |
| `author` | No | string | Course author. Defaults to `Anonymous`. |
| `version` | No | string | Content version. Defaults to `0.0.0`. |
| `quests` | Yes | list | List of quests in this course. |

Recommended:

```yaml
title: "Module 1: Reactivation"
author: "Supershell Team"
version: "1.0.0"
quests: []
```

### Legacy Loading Note

The engine can still load older YAML shapes, including a top-level list of quests or a single quest object. New content should use the full course format.

Use:

```yaml
title: "..."
author: "..."
version: "..."
quests:
  - id: "..."
```

Avoid adding new content in older formats.

## Quest Fields

Each quest belongs under `quests`.

| Field | Required | Type | Purpose |
|---|---:|---|---|
| `id` | Yes | string | Stable internal quest identifier. |
| `title` | Yes | string | Player-facing quest title. |
| `construct` | No | boolean | Whether this quest uses the Construct world. Defaults to `true`. |
| `chapters` | Yes | list | Ordered list of chapters. |

Example:

```yaml
quests:
  - id: "reactivation"
    title: "Cold Start"
    construct: true
    chapters: []
```

### Quest IDs

Use short, stable, lowercase identifiers:

```yaml
id: "reactivation"
id: "maintenance_deck"
id: "archive_recovery"
```

Avoid changing quest IDs after release because saved progress may depend on them.

## Chapter Fields

Each chapter belongs under a quest's `chapters` list.

| Field | Required | Type | Purpose |
|---|---:|---|---|
| `title` | Yes | string | Chapter title shown to the player. |
| `intro` | Yes | string | Narrative/instruction text shown before tasks. |
| `outro` | Yes | string | Text shown after chapter completion. |
| `setup_actions` | No | list | World-building actions for the Construct. Defaults to empty. |
| `tasks` | Yes | list | Ordered list of tasks. |

Example:

```yaml
chapters:
  - title: "Emergency Light"
    intro: |
      Operator, your console is active.
      Use `ls` to scan the current place.

    setup_actions:
      - type: ResetWorld
      - type: CreateDir
        path: "Memory_Bank"

    tasks:
      - objective: "Scan the Training Area"
        description: "Use `ls` to scan your current place."
        instruction: "Run `ls`."
        success_msg: "Scan complete."
        conditions:
          - type: CommandMatches
            pattern: '^ls(\s.*)?$'

    outro: |
      Scan complete. A new place is visible.
```

## Task Fields

Each task belongs under a chapter's `tasks` list.

| Field | Required | Type | Purpose |
|---|---:|---|---|
| `objective` | Yes | string | Short objective title shown to the player. |
| `description` | Yes | string | Brief explanation of what the task teaches. |
| `instruction` | Yes | string | Direct player-facing instruction. |
| `success_msg` | Yes | string | Message shown when the task completes. |
| `hint` | No | string | Optional hint text. Defaults to empty. |
| `conditions` | Yes | list | Conditions that must pass for completion. |
| `rewards` | No | list | Optional state changes applied on completion. Defaults to empty. |

Example:

```yaml
tasks:
  - objective: "Enter the Memory Bank"
    description: "Move into the `Memory_Bank` directory."
    instruction: "Run `cd Memory_Bank`."
    success_msg: "Location changed."
    hint: "Use `cd` followed by the place name."
    conditions:
      - type: CommandMatches
        pattern: '^cd\s+Memory_Bank/?\s*$'
      - type: WorkingDir
        path: "Memory_Bank$"
```

## Condition Model

Conditions determine whether a task is complete.

A task should usually include one command condition and, when needed, one or more logic/context conditions.

Example:

```yaml
conditions:
  - type: CommandMatches
    pattern: '^cd\s+Memory_Bank/?\s*$'
  - type: WorkingDir
    path: "Memory_Bank$"
```

In this example:

- `CommandMatches` checks what the player typed.
- `WorkingDir` checks where the shell ended up after the command.

This distinction matters because Supershell should not advance when a command text looks right but the real command failed.

## Command Conditions

### `CommandMatches`

Checks whether the player's command matches a regular expression.

```yaml
- type: CommandMatches
  pattern: '^ls(\s.*)?$'
```

Use this for command syntax.

Good examples:

```yaml
pattern: '^ls(\s.*)?$'
pattern: '^cd\s+Memory_Bank/?\s*$'
pattern: '^cat\s+welcome_packet\.txt\s*$'
```

Tips:

- Anchor patterns with `^` and `$`.
- Escape literal dots, such as `welcome_packet\.txt`.
- Allow natural variations when helpful, such as an optional trailing slash for directories.
- Do not make patterns so loose that unrelated commands pass.

### `HistoryContains`

Checks whether command history contains a matching pattern.

```yaml
- type: HistoryContains
  pattern: 'ls'
```

Avoid relying on this in new content until history behavior is intentionally reviewed and tested.

## Construct Path Conditions

These conditions check files and directories inside the Construct.

Construct paths must be relative paths. They must not be absolute paths, empty paths, or paths using `..` to escape the Construct.

### `PathExists`

```yaml
- type: PathExists
  path: "Memory_Bank/Sector_A/welcome_packet.txt"
```

Passes when the path exists.

### `PathMissing`

```yaml
- type: PathMissing
  path: "temporary_marker.txt"
```

Passes when the path does not exist.

### `IsDirectory`

```yaml
- type: IsDirectory
  path: "Memory_Bank"
```

Passes when the path exists and is a directory.

### `IsFile`

```yaml
- type: IsFile
  path: "Memory_Bank/Sector_A/welcome_packet.txt"
```

Passes when the path exists and is a file.

### `IsExecutable`

```yaml
- type: IsExecutable
  path: "scripts/repair.sh"
```

Passes when the path exists and has executable permissions.

This is Unix-oriented behavior. Be careful with cross-platform lessons.

## File Content Conditions

### `FileContains`

```yaml
- type: FileContains
  path: "notes.txt"
  pattern: "operator"
```

Passes when the file contains text matching the regular expression.

### `FileNotContains`

```yaml
- type: FileNotContains
  path: "notes.txt"
  pattern: "temporary"
```

Passes when the file does not contain text matching the regular expression.

### `FileEmpty`

```yaml
- type: FileEmpty
  path: "blank_marker.txt"
```

Passes when the file exists and has zero bytes.

## Environment Conditions

### `WorkingDir`

Checks the player's current working directory.

```yaml
- type: WorkingDir
  path: "Memory_Bank$"
```

Use this with `cd` tasks.

Recommended pattern style:

```yaml
path: "Memory_Bank$"
path: "Sector_A$"
```

This lets the condition match the end of the current path without hard-coding the user's full machine-specific path.

### `EnvVar`

Checks an environment variable.

```yaml
- type: EnvVar
  name: "SUPERSHELL_TEST_MODE"
  value: "1"
```

Use sparingly in player-facing content.

## Game State Conditions

These conditions check Supershell state rather than the filesystem.

### `FlagIsTrue`

```yaml
- type: FlagIsTrue
  key: "scanner_enabled"
```

Passes when the named flag is true.

### `VarEquals`

```yaml
- type: VarEquals
  key: "calibration_level"
  value: 3
```

Passes when the named integer variable equals the target value.

### `VarGreaterThan`

```yaml
- type: VarGreaterThan
  key: "signals_found"
  value: 2
```

Passes when the named integer variable is greater than the target value.

### `VarLessThan`

```yaml
- type: VarLessThan
  key: "errors_remaining"
  value: 1
```

Passes when the named integer variable is less than the target value.

## Condition Failure Messages

Conditions may include an optional `failure_message`.

```yaml
conditions:
  - type: WorkingDir
    path: "Memory_Bank$"
    failure_message: "The command looked right, but you are not inside the Memory_Bank yet."
```

Use failure messages for logic/context failures, especially when the command syntax was correct but the world state is wrong.

Good failure messages should:

- explain what did not happen
- avoid blaming the player
- suggest what to inspect next
- stay short

Example:

```yaml
failure_message: "The place exists, but you are not standing inside it yet. Try checking your current location."
```

## Rewards

Rewards update Supershell's internal game state when a task completes.

Supported reward types:

- `SetFlag`
- `SetVar`
- `AddVar`

### `SetFlag`

```yaml
rewards:
  - type: SetFlag
    key: "scanner_enabled"
    value: true
```

Sets a boolean flag.

### `SetVar`

```yaml
rewards:
  - type: SetVar
    key: "signals_found"
    value: 1
```

Sets an integer variable.

### `AddVar`

```yaml
rewards:
  - type: AddVar
    key: "signals_found"
    amount: 1
```

Adds to an integer variable.

## Setup Actions

Setup actions prepare the Construct world.

They belong under a chapter:

```yaml
setup_actions:
  - type: ResetWorld
  - type: CreateDir
    path: "Memory_Bank"
```

Setup actions currently run from the Construct root.

### `ResetWorld`

```yaml
- type: ResetWorld
```

Clears the current Construct contents.

Use this at the start of an intro or scenario when the lesson needs a known clean world.

### `CreateDir`

```yaml
- type: CreateDir
  path: "Memory_Bank"
```

Creates a directory.

Nested directories are allowed:

```yaml
- type: CreateDir
  path: "Memory_Bank/Sector_A"
```

### `CreateFile`

```yaml
- type: CreateFile
  path: "Memory_Bank/Sector_A/welcome_packet.txt"
  content: |
    ARK WELCOME PACKET
    ------------------

    Operator, your console is responding.
```

Creates a file and writes content.

### `RemovePath`

```yaml
- type: RemovePath
  path: "temporary_marker.txt"
```

Removes a file or directory if it exists.

Use carefully. Prefer `ResetWorld` at the beginning of a controlled lesson over many scattered removals.

## Path Safety Rules

Any content-authored path that touches the Construct should be a safe relative path.

Allowed:

```yaml
path: "Memory_Bank"
path: "Memory_Bank/Sector_A"
path: "Memory_Bank/Sector_A/welcome_packet.txt"
```

Rejected:

```yaml
path: ""
path: "/tmp/outside.txt"
path: "../outside.txt"
path: "Memory_Bank/../../outside.txt"
```

Content should never require writing outside the Construct.

## Authoring Style Guidelines

Good Supershell content should:

- teach one new concept at a time
- preserve real shell behavior
- keep commands real
- avoid fake game-only commands
- use clear objectives
- give short, actionable hints
- reward observation
- avoid destructive commands until taught carefully
- make failure recoverable
- keep story text immersive but not excessive

## Recommended Task Pattern

Use this shape for most beginner tasks:

```yaml
- objective: "Short Objective"
  description: "Explain what concept this task teaches."
  instruction: "Tell the player exactly what command to try."
  success_msg: "Confirm what changed."
  hint: "Optional extra guidance."
  conditions:
    - type: CommandMatches
      pattern: '^command\s+argument\s*$'
```

For movement tasks, add `WorkingDir`:

```yaml
- objective: "Enter Sector A"
  description: "Move into `Sector_A`."
  instruction: "Run `cd Sector_A`."
  success_msg: "Location changed. You are inside `Sector_A`."
  conditions:
    - type: CommandMatches
      pattern: '^cd\s+Sector_A/?\s*$'
    - type: WorkingDir
      path: "Sector_A$"
```

For file creation tasks, combine command and filesystem checks:

```yaml
- objective: "Create a Marker"
  description: "Create a blank marker file."
  instruction: "Run `touch marker.txt`."
  success_msg: "Marker created."
  conditions:
    - type: CommandMatches
      pattern: '^touch\s+marker\.txt\s*$'
    - type: IsFile
      path: "marker.txt"
```

For file content tasks, combine command and content checks:

```yaml
- objective: "Record a Signal"
  description: "Write the signal word into a file."
  instruction: "Use `echo` and redirection to write `relay` into `signal.txt`."
  success_msg: "Signal recorded."
  conditions:
    - type: CommandMatches
      pattern: '^echo\s+relay\s+>\s+signal\.txt\s*$'
    - type: FileContains
      path: "signal.txt"
      pattern: '^relay\s*$'
```

## Regular Expression Tips

Use regex patterns carefully.

### Anchor Command Patterns

Prefer:

```yaml
pattern: '^ls(\s.*)?$'
```

Avoid:

```yaml
pattern: 'ls'
```

The loose version could match unrelated commands.

### Escape Literal Dots

Prefer:

```yaml
pattern: '^cat\s+welcome_packet\.txt\s*$'
```

Avoid:

```yaml
pattern: '^cat\s+welcome_packet.txt\s*$'
```

An unescaped `.` means "any character" in regex.

### Allow Natural Directory Syntax

For `cd` tasks, allow a trailing slash:

```yaml
pattern: '^cd\s+Memory_Bank/?\s*$'
```

This accepts both:

```bash
cd Memory_Bank
cd Memory_Bank/
```

## Common Mistakes

### Mistake: Only checking command text for `cd`

Weak:

```yaml
conditions:
  - type: CommandMatches
    pattern: '^cd\s+Memory_Bank/?\s*$'
```

Better:

```yaml
conditions:
  - type: CommandMatches
    pattern: '^cd\s+Memory_Bank/?\s*$'
  - type: WorkingDir
    path: "Memory_Bank$"
```

Why:

The command text might look right, but the command could still fail. The task should verify the real resulting location.

### Mistake: Hard-coding a full local path

Weak:

```yaml
- type: WorkingDir
  path: "/Users/example/Construct/Memory_Bank"
```

Better:

```yaml
- type: WorkingDir
  path: "Memory_Bank$"
```

Why:

Students and test machines have different home directories.

### Mistake: Unsafe Construct paths

Never use:

```yaml
path: "../outside.txt"
path: "/tmp/file.txt"
```

Use relative Construct paths only:

```yaml
path: "Memory_Bank/file.txt"
```

### Mistake: Long lore before the first command

Weak:

```yaml
intro: |
  Several pages of story before the student knows what to do...
```

Better:

```yaml
intro: |
  Operator, your console is active.
  Use `ls` to scan the current place.
```

The first few lessons should be short, clear, and interactive.

## Validation

Validate a course with:

```bash
cargo run -- --validate library/intro.yaml
```

Before committing content changes, run the full project gate:

```bash
cargo fmt --check
cargo check
cargo test
cargo run -- --validate library/intro.yaml
```

## Content Commit Checklist

Before opening a PR:

```bash
git status -sb
cargo fmt --check
cargo check
cargo test
cargo run -- --validate library/intro.yaml
```

Check the content manually:

- Does every task have a clear objective?
- Does every task have a clear instruction?
- Are regexes anchored?
- Are file paths relative?
- Are `cd` tasks checked with `WorkingDir`?
- Does the story support the shell concept being taught?
- Does the content avoid fake shell behavior?
- Does the lesson feel fair to a beginner?

## Current v0.5 Schema Summary

```text
Course
├── title
├── author
├── version
└── quests
    └── Quest
        ├── id
        ├── title
        ├── construct
        └── chapters
            └── Chapter
                ├── title
                ├── intro
                ├── setup_actions
                ├── tasks
                │   └── Task
                │       ├── objective
                │       ├── description
                │       ├── instruction
                │       ├── success_msg
                │       ├── hint
                │       ├── conditions
                │       └── rewards
                └── outro
```

Supported setup actions:

```text
ResetWorld
CreateDir
CreateFile
RemovePath
```

Supported rewards:

```text
SetFlag
SetVar
AddVar
```

Supported conditions:

```text
CommandMatches
HistoryContains
PathExists
PathMissing
IsDirectory
IsFile
IsExecutable
FileContains
FileNotContains
FileEmpty
WorkingDir
EnvVar
FlagIsTrue
VarEquals
VarGreaterThan
VarLessThan
```
