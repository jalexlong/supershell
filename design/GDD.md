# Supershell: Game Design Document

This document describes the intended player experience and design constraints.
For narrative details (world, characters, story arc) see `design/narrative.md`.
For technical implementation see `design/architecture.md`.

---

## 1. High Concept

**Supershell** is a terminal-based narrative RPG for middle schoolers with no prior
CLI experience. The player learns real bash commands by using them to solve problems
inside a fantasy kingdom called the Construct.

The game never names a command for the player. Instead, the narrative creates a
situation where the action is obvious — and the player discovers the command themselves.

**Target audience:** Middle school students, no technical background assumed.

**Core design rule:** The technical serves the narrative. A lesson that requires an
awkward story beats the story every time.

---

## 2. Core Loop

```
1. Briefing   — quest card shows the objective (no command named)
2. Action     — player types a real bash command in the live session
3. Validation — alias interceptor pipes the command to supershell --check
4. Reaction:
     Success  — green status card, next objective
     Failure  — glitch effect on the error message; hint after 3 failures
     Neutral  — unrelated commands pass through silently
```

The player's normal terminal behavior is preserved throughout. The game is a layer,
not a replacement.

---

## 3. Mechanics

### 3.1 Knowledge-Gated Progression

There are no stat unlocks. Every tool is available from the start. The game's
challenge is the player realizing they *need* a specific tool — the narrative
provides the why, the player figures out the how.

Skills unlock the *world*: `ls -a` reveals hidden areas, `chmod` makes NPCs
executable (marking available quests). Nothing is gated behind levels or points.

### 3.2 Failure: The Glitch Effect

The player cannot die or lose progress. Failure is visual: error messages render
with Unicode combining strikethrough (U+0336), producing the text-corruption look.

After three consecutive failures on the same task, the task's `hint` field surfaces
as a "Corrupted Data Fragment." The failure count resets on success and persists
across restarts. *(Implemented: M5)*

### 3.3 World Destruction Recovery

If the player destroys the Construct (e.g., `rm -rf ~/Construct`), the engine
detects the missing root on the next command and performs an instant "System
Restore" — recreating the directory and rerunning the current chapter's setup
actions. *(Implemented: M6)*

### 3.4 NPC System

NPCs are files inside the Construct. Reading one with `less` shows:

- The NPC's voice (first person, direct address — what they'd say to the reader)
- A bare hex hash on the final line — no label, no explanation

The hash is each NPC's identity fingerprint. When the Glitch corrupts an NPC, the
hash changes. The player is never told this explicitly — they discover it by noticing.

An NPC with a quest available is marked executable, visible only via `ls -la`. This
makes reading permissions a skill that literally opens the world.

### 3.5 Quest Structure

```
Lesson Quest          introduces a new command through narrative walkthrough
  Mandatory Drill 1   less guidance; hints available after 3 failures
  Mandatory Drill 2
  [Optional] Quest A  labeled optional; lore-heavy or silly; safe to skip
  [Optional] Quest B
Next Lesson Quest
```

---

## 4. UI Constraints

- **No HUD.** The player uses their standard terminal prompt.
- **No clearing the screen** except at chapter transitions.
- **Success:** clean white/green box-drawing card.
- **Failure:** red glitch effect on the error message.
- **Hint:** yellow, after three consecutive failures.
- **Irrelevant commands:** completely silent — no game output, no card.

The UI must degrade gracefully in constrained terminals (narrow width, no raw mode).
A failed render should never make the shell unusable.

---

## 5. Implemented vs. Planned

| Feature | Status |
|---------|--------|
| Transient bash session with alias interceptor | Done |
| Two-pass command validation (relevance → logic) | Done |
| YAML quest schema (`Course → Quest → Chapter → Task`) | Done |
| Save/load with atomic write | Done |
| Glitch effect on failure | Done (M6) |
| Hint system after 3 failures | Done (M5) |
| World-destruction auto-restore | Done (M6) |
| Interactive module menu | Done (M3) |
| `HistoryContains` condition | Done (M4) |
| Constrained-terminal fallback | Done |
| Tutorial module | Done (v0.5.8) |
| NPC manifest system (`less NPC`) | Content only — no engine changes needed |
| Executable-bit NPC quest markers | Planned (v0.6.0) |
| Quest prerequisites | Planned (v0.6.0) |
| Optional quest flag | Planned (v0.6.0) |
| Hash corruption detection | Planned (v0.8.0 — Act 2c) |
| Quest editor GUI | Planned (v1.0.0) |
