# Supershell

A narrative RPG that runs in your terminal and teaches real shell skills through story.

You wake up at the edge of the Construct with no memory. A companion named Bit finds
you — slightly out of breath, clearly worried — and pulls you toward Stonehaven.
Something is rewriting the world from the inside. You're the Operator. You're the
only one who can do anything about it.

Every action you take is a real bash command. The game watches, validates, and responds.
You learn `ls`, `cd`, `cat`, `grep`, `chmod` not from a tutorial — from needing them.

---

## Current Content

Supershell is in active development. The current release includes:

- **Tutorial** — narrative cold open; `ls` and `cd` discovered organically through story
- **Intro module** — guided orientation arc; `ls`, `cd`, `cat`
- **Permissions module** — access control arc; `ls -la`, `chmod +x`, octal permissions
- Save persistence, hint system after 3 failures, glitch failure effect, auto world-restore

The game is playable end-to-end for the content covered so far. The full three-act
story is in active development — see the roadmap below.

---

## Install

```bash
cargo install supershell
```

## Running

```bash
supershell           # launch the Construct
supershell --status  # show current objective
supershell --menu    # switch modules
supershell --reset   # wipe save and start over
```

---

## How It Works

Supershell launches a transient bash session with a lightweight alias interceptor.
Every command you type is validated against the current quest objective in the Rust
binary. Your real shell is untouched — the session is temporary and exits cleanly.

Progress saves automatically. Exit anytime with `exit`.

---

## The World

The game takes place inside `~/Construct` — a fantasy kingdom with clearly digital bones.

```
~/Construct/
├── Stonehaven/       home base; market, inn, tavern
├── RoyalLibrary/     the great archive
├── Hammerstone/      craftsmen's quarter
├── Deepwood/         the wild forest
├── Greyspire/        seat of power
├── .Misthollow/      hidden — requires ls -a
└── TheShatter/       Act 3: corrupted territory
```

NPCs are files. Reading one with `less` shows their current state — their voice,
what they need, and an unexplained hex string at the bottom. When that string
changes, something has gone wrong.

Hidden areas (prefixed with `.`) are only visible once you know to look for them.

---

## Roadmap

| Milestone | Content |
|-----------|---------|
| Current (v0.5.8) | Tutorial, intro, and permissions modules; engine stabilization complete |
| Act 1 | Stonehaven arc — `cat`, `pwd`, `less`, `head`, `tail`; full NPC quest board |
| Act 2a | Hammerstone/Deepwood — `mkdir`, `touch`, `cp`, `mv`, `rm` |
| Act 2b | RoyalLibrary — `grep`, `find`, pipes, redirection |
| Act 2c | Greyspire — `ls -la`, `chmod`, `sudo`; hidden areas; hash corruption mechanic |
| Act 3 | TheShatter — confrontation with The Glitch |
| v1.0.0 | Full game complete + quest editor GUI for educators |

---

## Developing

```bash
cargo fmt --check && cargo check && cargo test
```

Quest content lives in `library/*.yaml`. Validate a module with:

```bash
supershell --validate library/yourmodule.yaml
```

See [`CLAUDE.md`](CLAUDE.md) for architecture and schema reference.
See [`design/narrative.md`](design/narrative.md) for the story design document.

---

## License

MIT
