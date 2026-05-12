# Supershell Content Roadmap

## Purpose

This roadmap defines the planned learning sequence for Supershell content.

Supershell teaches real shell usage through the Derelict Data Ark story framework.

## Design Rules

- Real shell behavior first, game overlay second.
- Directories are places.
- Files are objects, items, or things.
- Commands are actions.
- Each module should teach a small number of related shell concepts.
- Early modules should provide strong training wheels.
- Later modules should ask students to infer more from the environment.

## Module 1: Reactivation

Status: Implemented

Commands:

- `ls`
- `cd`
- `cat`

Learning goals:

- Understand that the shell has a current location.
- Use `ls` to inspect a place.
- Use `cd` to move between places.
- Understand that files are different from directories.
- Use `cat` to read a text file.

Story role:

The Operator wakes aboard the Ark and restores basic console awareness.

## Module 2: Orientation

Status: Planned

Commands:

- `pwd`
- `ls -l`
- `clear`

Learning goals:

- Check the current location with `pwd`.
- Read long listings with `ls -l`.
- Recognize file/directory metadata.
- Clear the terminal without changing state.

Story role:

The Operator learns to read location and object metadata inside the Ark.

## Module 3: Maintenance Deck

Status: Planned

Commands:

- `mkdir`
- `touch`
- `cp`
- `mv`

Learning goals:

- Create directories.
- Create files.
- Copy files.
- Move and rename files.

Story role:

The Operator repairs a damaged maintenance area by creating and organizing system objects.

## Module 4: Careful Deletion

Status: Planned

Commands:

- `rm`
- `rm -i`
- possibly `rmdir`

Learning goals:

- Understand deletion risk.
- Delete files intentionally.
- Avoid destructive commands without checking location.

Story role:

The Operator clears corrupted temporary objects while following safety protocols.

## Module 5: Archive Recovery

Status: Planned

Commands:

- `grep`
- `head`
- `tail`
- `wc`

Learning goals:

- Search inside files.
- Inspect large logs.
- Count lines, words, and bytes.
- Extract clues from text.

Story role:

The Operator searches recovered logs to reconstruct what happened aboard the Ark.

## Module 6: Signal Routing

Status: Planned

Commands:

- `|`
- `>`
- `>>`

Learning goals:

- Pipe output between commands.
- Redirect output to files.
- Append output safely.

Story role:

The Operator routes diagnostic signals between damaged Ark subsystems.

## Module 7: Access Control

Status: Planned

Commands:

- `ls -l`
- `chmod`

Learning goals:

- Read permission bits.
- Understand executable files.
- Change permissions carefully.

Story role:

The Operator restores access to locked maintenance scripts.

## Module 8: Remote Relay

Status: Future

Commands:

- `ping`
- `ssh`
- `scp`

Learning goals:

- Understand remote systems.
- Connect responsibly.
- Transfer files.

Story role:

The Operator establishes communication with a remote Ark relay.

## Open Questions

- Should `pwd` appear in Module 1 or Module 2?
- Should deletion be taught before or after copying/moving?
- How early should permissions appear?
- Should networking wait until after local filesystem mastery?
- What is the first “mystery” that spans multiple modules?

## Current Priority

Next implemented content should probably be Module 2: Orientation.
