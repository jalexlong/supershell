# Supershell Playtesting Guide

This document describes the manual playtest process for the current v0.5 intro module.

Automated tests verify schema validity, state behavior, path safety, and basic command progression. Manual playtesting verifies the actual user experience: pacing, readability, command flow, and whether the lesson feels good in a real terminal.

## Before Playtesting

Supershell may reset the Construct directory:

```text
~/Construct
```

If that directory contains anything important, back it up first:

```bash
mv ~/Construct ~/Construct.backup.$(date +%Y%m%d-%H%M%S)
```

## Standard Quality Gate

Run this before a manual playtest:

```bash
cargo fmt --check
cargo check
cargo test
```

All checks should pass with no warnings.

## Validate Intro Content

Run:

```bash
cargo run -- --validate library/intro.yaml
```

Expected result:

```text
>> [SUCCESS] YAML Syntax is valid.
>> Title:   Module 1: Reactivation
>> Author:  Supershell Team
>> Version: 1.0.0
```

## Reset Progress

Run:

```bash
cargo run -- --reset --status
```

Expected result:

- save data is wiped
- the intro module loads
- the first task is shown
- the active objective is `Scan the Training Area`

## Launch the Guided Shell

Run:

```bash
cargo run
```

This starts the guided Supershell session.

## Expected Intro Walkthrough

Run these commands inside the guided shell.

### 1. Scan the Training Area

```bash
ls
```

Expected:

- task completes
- success message appears
- next objective becomes `Enter the Memory Bank`

### 2. Enter the Memory Bank

```bash
cd Memory_Bank
```

Expected:

- task completes immediately
- success message appears
- Supershell recognizes the new working directory on the first attempt

### 3. Scan the Memory Bank

```bash
ls
```

Expected:

- task completes
- `Sector_A` is visible
- next objective becomes `Enter Sector A`

### 4. Enter Sector A

```bash
cd Sector_A
```

Expected:

- task completes immediately
- success message appears
- Supershell recognizes the new working directory on the first attempt

### 5. Read the Welcome Packet

```bash
cat welcome_packet.txt
```

Expected:

- file contents display
- task completes
- final outro appears
- module completion is saved

## Verify Completion

After completing the module, run:

```bash
cargo run -- --status
```

Expected:

```text
>> [SYSTEM] Quest Complete. Run 'supershell --menu' for more.
```

## Verify Construct Contents

Run:

```bash
find ~/Construct -maxdepth 4 -type f -o -type d | sort
```

Expected structure:

```text
~/Construct
~/Construct/Memory_Bank
~/Construct/Memory_Bank/Sector_A
~/Construct/Memory_Bank/Sector_A/welcome_packet.txt
```

Then run:

```bash
cat ~/Construct/Memory_Bank/Sector_A/welcome_packet.txt
```

Expected:

- the welcome packet text from `library/intro.yaml` is present

## Constrained Terminal Check

Test once in a short or embedded terminal pane, such as an editor terminal.

Expected:

- Supershell falls back to plain text rendering
- no bordered UI overlaps itself
- text remains readable
- the shell remains usable

## Playtest Notes

During playtesting, record:

- unclear instructions
- awkward pacing
- missing feedback
- confusing command behavior
- rendering problems
- places where a student might get stuck

The goal is not only to check that the module works, but that it teaches clearly.
