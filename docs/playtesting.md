# Supershell Playtesting Guide

This document describes the manual playtest process for all current modules.

Automated tests verify schema validity, state behavior, path safety, and command progression. Manual playtesting verifies the actual user experience: pacing, readability, command flow, and whether the lesson feels good in a real terminal.

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

## Validate Module Content

```bash
cargo run -- --validate library/intro.yaml
cargo run -- --validate library/permissions.yaml
```

Expected result for each:

```text
>> [SUCCESS] YAML Syntax is valid.
>> Title:   <module title>
>> Author:  Supershell Team
>> Version: 1.0.0
```

## Reset Progress

```bash
cargo run -- --reset --status
```

Expected:

- save data is wiped
- the intro module loads
- the first task is shown: `Activate Visual Sensors`

---

## Module 1: Awakening (intro.yaml)

Launch the guided shell:

```bash
cargo run
```

### Golden Path

| # | Command | Expected outcome |
|---|---------|-----------------|
| 1 | `ls` | Task completes; objective becomes `Enter the Memory Bank` |
| 2 | `cd Memory_Bank` | Task completes on first attempt; objective becomes `Scan the Room` |
| 3 | `ls` | Task completes; `Sector_A` visible; objective becomes `Enter Sector A` |
| 4 | `cd Sector_A` | Task completes on first attempt; objective becomes `Read the Data Packet` |
| 5 | `cat welcome_packet.txt` | File contents shown; final outro plays; module marked complete |

### Failure-Path Tests

**Wrong command (irrelevant):**
- While on the `ls` task, type `pwd`
- Expected: command output shown normally, no game overlay, no status card

**Repeated failure triggering hint:**
- Edit `save.json` to point at chapter 0, task 1 (cd Memory_Bank) with `failure_count: 2`
- Type `cd wrong_dir` (matches `^cd\s+Memory_Bank\s*$`? No — actually type something that *would* match the pattern but fail the WorkingDir logic condition, e.g. set cwd to wrong dir via `--check`)
- Simpler: run `cargo run -- --check "cd Memory_Bank" --cwd "/tmp"` three times
- On the third run, expected: hint appears in yellow below the failure card

**Glitch effect:**
- Trigger any logic failure (see above)
- Expected: the error message on the `└──` line has visible strikethrough on every character

### Verify Construct Contents

```bash
find ~/Construct -maxdepth 4 | sort
```

Expected:

```text
~/Construct
~/Construct/Memory_Bank
~/Construct/Memory_Bank/Sector_A
~/Construct/Memory_Bank/Sector_A/welcome_packet.txt
```

---

## Module 2: Access Control (permissions.yaml)

Switch to the permissions module:

```bash
cargo run -- --menu   # select permissions.yaml
cargo run
```

### Golden Path

| # | Command | Expected outcome |
|---|---------|-----------------|
| 1 | `ls -la mission_files` | Task completes; `matrix_read` flag set |
| 2 | `cat mission_files/access.log` | File contents shown; task completes |
| 3 | `chmod +x deploy.sh` | `deploy.sh` becomes executable; `exec_granted` flag set |
| 4 | `chmod 600 private.key` | Permissions set to owner-read-only; module complete |

### Failure-Path Tests

**Flag-gated task blocked:**
- While on chapter 1 task 2 (`cat access.log`), manually clear flags from `save.json`
- Type `cat mission_files/access.log`
- Expected: logic failure with glitch effect; hint appears after 3 consecutive failures

**Wrong chmod syntax:**
- On the `chmod +x` task, type `chmod 755 deploy.sh`
- Expected: command runs normally (irrelevant — regex requires `+x` form) with no game response

---

## World-Destruction Recovery

From inside any active session:

1. Open a second terminal and run `rm -rf ~/Construct`
2. Back in the game session, type any hooked command (e.g. `ls`)
3. Expected output:
   ```text
   >> [SYSTEM] Construct corruption detected. Restoring...
   >> [SYSTEM] Reconfiguring Construct...
   ```
4. The Construct directory should be recreated and the session continues normally
5. Verify with `find ~/Construct -maxdepth 3 | sort` in the second terminal

---

## Constrained Terminal Check

Test in a short or narrow terminal pane (e.g. a split editor terminal).

Expected:

- Supershell falls back to plain text rendering
- no bordered UI overlaps itself
- text remains readable
- the shell remains usable

---

## Verify Module Completion

After completing any module:

```bash
cargo run -- --status
```

Expected:

```text
>> [SYSTEM] Quest Complete. Run 'supershell --menu' for more.
```

---

## Playtest Notes

During playtesting, record:

- unclear instructions
- awkward pacing
- missing feedback
- confusing command behavior
- rendering problems
- places where a student might get stuck

The goal is not only to check that the module works, but that it teaches clearly.
