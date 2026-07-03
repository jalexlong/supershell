# TODO — Supershell Development Backlog

Engine milestones M1–M8 are complete as of v0.5.8. Items below are grouped by
what's needed next. Each content milestone is a patch bump; v1.0.0 is the first
release with a full story and quest editor.

---

## Content — Act 1: Stonehaven (next)

Teaches: `cat`, `pwd`, `less`, `head`, `tail`

- [ ] Design 3–4 Stonehaven NPCs (names, manifest voice, quest hooks)
- [ ] Write lesson quest: `cat` — a document in the RoyalLibrary needs reading
- [ ] Write lesson quest: `less` — a longer scroll; `cat` isn't enough
- [ ] Write lesson quest: `pwd` — Bit needs to know exactly where the Operator is
- [ ] Write mandatory drill quests for each command
- [ ] Build `Stonehaven/` world structure in YAML setup actions
- [ ] First signs of corruption: an NPC's hash quietly changes mid-arc

Engine work needed for Act 1:
- [ ] Quest prerequisites — a quest requires another to be completed first
- [ ] Optional quest flag — player sees the quest but is not blocked by skipping it
- [ ] Executable bit condition (`IsExecutable` exists; wire it to NPC quest state in setup actions)

---

## Content — Act 2a: Hammerstone / Deepwood

Teaches: `mkdir`, `touch`, `cp`, `mv`, `rm`

- [ ] Write the Goblin Under the Bridge quest (`mv` vs `rm` lesson)
- [ ] Write the NotSoHiddenNinja quest (`mv` as rename)
- [ ] Build `Hammerstone/` and `Deepwood/` world structures
- [ ] Corruption becomes visible — NPCs behaving strangely, manifests altered

---

## Content — Act 2b: RoyalLibrary

Teaches: `grep`, `find`, `|` pipes, `>` and `>>` redirection

- [ ] Write lesson quest: find a corrupted record in the archive via `grep`
- [ ] Build `RoyalLibrary/` world structure with readable documents
- [ ] The Glitch is identified — former archivist process of the RoyalLibrary

---

## Content — Act 2c: Greyspire

Teaches: `ls -la`, `chmod`, `sudo`

- [ ] The `ls -a` unlock moment — hidden areas become visible for the first time
- [ ] Write lesson quest: `chmod` grants an NPC the ability to speak (executable bit)
- [ ] The sudo moment — narrative event that grants operator-level access
- [ ] Hash corruption mechanic — player notices changed hashes; engine detects and responds

Engine work needed for Act 2c:
- [ ] Hash integrity tracking — store expected NPC hashes in state; detect changes
- [ ] Hidden area reveal — setup actions can toggle dotfile prefix on directories

---

## Content — Act 3: TheShatter

Teaches: process basics (`ps`, background jobs), env vars, scripting intro

- [ ] Build `TheShatter/` — corrupted structure, garbled names, wrong permissions
- [ ] Write Glitch confrontation — articulate antagonist with a coherent worldview
- [ ] Write resolution paths (restore / contain / remove)
- [ ] Final scene with Bit

---

## Engine Backlog

- [ ] **Quest prerequisites** — `requires: [quest_id]` in YAML; engine skips ineligible quests
- [ ] **Optional quest flag** — `optional: true` in YAML; shown in quest board but not blocking
- [ ] **Score & replay** — per-task completion time + failure count; `--score` summary card; allow replaying completed modules from `--menu`
- [ ] **Hash corruption detection** — store NPC hashes in `GameState`; `FileHashChanged` condition type
- [ ] **Quest editor GUI** — visual tool for educators; form-driven YAML authoring; live preview; `--validate` integration (v1.0.0)

---

## Known Quirks

- **`HistoryContains` timing** — `history -a` flushes after the command runs, so the matched command isn't visible until the next invocation. Design `HistoryContains` tasks so the check fires on a follow-up command.
