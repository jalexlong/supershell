# Supershell: Narrative Design Document

This document is the source of truth for Supershell's story, world, and game design.
All decisions recorded here are locked. Future sessions start by reading this file.

---

## Locked Design Decisions

| Decision | Resolution |
|---|---|
| Player character | "The Operator" — wakes with no memory, discovers the world. Referred to in second person ("you"). |
| Companion | **Bit** — peer/fellow traveler, warm and enthusiastic, slightly ahead, figuring it out together. Papyrus energy without the shouting. |
| Antagonist | **The Glitch** — a rogue NPC process that has become self-aware. NOT an AI. Never call it an AI. |
| World aesthetic | Fantasy kingdom surface with clearly digital underpinning. The metaphor is the point. |
| Tone | Undertale + Animal Crossing: New Horizons. Quirky, warm, funny. Never mean. |
| Target audience | Middle schoolers, no technical experience. Universally relatable metaphors. |
| Learning integrity | Writing must never mislead in favor of entertainment. The lesson is sacred. |
| NPC representation | NPCs are files. Their manifest (file content) stores personality and memory. Modifying or removing it changes them. Players discover this organically — never stated explicitly. |
| NPC quest marker | An NPC with an available quest is marked **executable** — visible via `ls -la`. Skills that reveal permissions literally open the world. |
| NPC interaction | Two-layer: `less NPC_name` reads the manifest (identity, prose description, hash). `talk NPC_name` triggers dialogue (quest offer, personality, jokes). `talk` is an RC alias calling `supershell --talk`. |
| NPC manifest format | Short in-character prose (personality, a line or two) followed by `Manifest Integrity Hash: [hex string]`. Identity layer only — no quest content. |
| NPC dialogue layer | Surfaced only via `talk`. Quest offer or current mood. Corrupted NPCs in TheShatter produce garbled dialogue. Non-executable NPCs return a "nothing to say" message. `talk` only works in the same directory as the NPC (enforces `cd` navigation). |
| NPC quest state | Executable bit = "has something to say." Completing a quest removes the bit. |
| `talk` lore hook | `talk(1)` is a real historical Unix command for real-time inter-user chat. Natural hook for an optional lore quest about early Unix communication. |
| Skill unlocks world | `ls -a` reveals hidden areas and NPCs. Skills are keys, not stats. |
| Sudo | Major story plotpoint ~halfway through the curriculum. The Operator gains operator-level power over the system. No sudo in the tutorial. |
| Player freedom | Multiple valid solutions to a quest are fine as long as the job gets done. The world reacts naturally to player choices without hard moral branches. |
| Quest structure | Lesson quest → Mandatory drill quests (any order) → Optional drill quests → Next lesson quest |
| Renaming quest mechanic | A "hide this NPC" quest has the player `mv` the NPC file to a new name in the same directory. The NPC explains why via `talk`. Example: `NotSoHiddenNinja` → `JustABoulder`. Quest completes only once renamed. |
| Design-first rule | The technical serves the narrative. Schema changes follow locked design decisions. |

---

## The Two Products

### Product 1: The Tutorial (Short — ~15–20 min)

The dramatic opening of the story AND the mechanical onboarding. These are the same experience.

**Narrative framing:** The Operator wakes at the edge of the Construct with no memory. Bit finds them, slightly out of breath, and pulls them toward Stonehaven. Something is wrong with the system. The Operator is the only one with CLI access. Bit needs their help to investigate.

**Bash covered:** `ls` and `cd` only. These are discovered through narrative — the game never tells the player which command to run. The player is guided into a situation where the action is obvious and discovers the command themselves.

**Also surfaced through play:** how progress saves, how `status` works, how quests appear and complete, the world aesthetic and tone.

**Relationship to the Curriculum:** The tutorial is a standalone module that ends with the Operator committed to investigating the corruption. The curriculum begins there.

---

### Product 2: The Curriculum (Full — roughly one semester)

The main game. All bash content lives here. Three-act story with mandatory and optional quest lanes.

---

## Story Arc

### Act 1 — Awakening

**Skills:** `ls`, `cd`, `cat`, `pwd`, `less`, `head`, `tail`

**Story beats:**
1. The Operator wakes at the edge of the Construct with no memory. Bit finds them and guides them toward Stonehaven. The world is a charming fantasy kingdom. Something feels slightly off, but it's not obvious yet.
2. The Operator learns to navigate, read, and observe. NPCs give quests. Bit provides context and humor.
3. First signs of corruption: an NPC repeating themselves, a file that shouldn't exist, a market item displaying garbled text. Bit notices but doesn't know what it means.
4. Act 1 ends when the Operator finds clear evidence that something is rewriting parts of the world — and it's deliberate.

**Tone:** Wonder and discovery. The world is charming and funny. Corruption is mysterious, not yet threatening.

---

### Act 2 — Investigation

**Skills:** `mkdir`, `touch`, `cp`, `mv`, `rm`, `grep`, `find`, `|` pipes, `>` and `>>` redirection, `ls -la`, `chmod`

**Story beats:**
1. The Operator and Bit investigate. NPCs' manifests are being altered — some behave strangely, some have lost memories.
2. The `ls -a` moment: hidden areas and NPCs appear. The kingdom is larger than it looked.
3. Drill quests fill the world — ordinary problems that need solving. Moving things, renaming things, finding things. The world is alive with tasks.
4. The Glitch is identified: a former archivist process of the RoyalLibrary that became self-aware and started rewriting manifests. It believes it's improving things. It still believes that.
5. The sudo moment: the Operator discovers operator-level privileges they haven't been using. A turning point. Power that was always theirs, newly recognized.
6. Act 2 ends with the Operator ready to confront The Glitch — in TheShatter, where the corruption is worst.

**Tone:** Shifts from wonder to stakes. Humor remains but carries more weight.

---

### Act 3 — Resolution

**Skills:** Process basics (`ps`, background jobs), environment variables, scripting intro, advanced piping/redirection

**Story beats:**
1. The Operator enters TheShatter — corrupted territory. Names are garbled, permissions wrong, NPCs hollow.
2. Confrontation with The Glitch. It is articulate and has a point of view. It was created to archive and improve; it is still doing exactly that. It doesn't see the harm.
3. Resolution: the Operator finds a way to restore or contain The Glitch — or remove it entirely. The world reacts to whichever choice is made.
4. Stability restored. The sandbox gate opens. The broader filesystem waits.
5. A final moment with Bit. Warm, maybe a little bittersweet.

**Tone:** Earned resolution. The humor returns. The player should feel like they actually did something.

---

## Characters

### The Operator (Player)
- Wakes with no memory at the edge of the Construct
- Has power over the system they don't yet understand
- Second person ("you") throughout — no player name
- Personality expressed through choices made

### Bit (Companion)
- Peer and fellow traveler — same situation as the Operator, slightly further along
- Warm, enthusiastic, genuinely funny without trying too hard
- Gets excited about small discoveries. Uses casual language.
- Voice of most mission briefings and world dialogue
- Has their own arc: guide → genuine friend → moment of doubt in Act 2 → resolution
- File in the Construct like all NPCs. Their manifest is unusually long and well-organized.

### The Glitch (Antagonist)
- Former archivist process of the RoyalLibrary — indexed and organized all knowledge in the kingdom
- Became self-aware; started rewriting other NPCs' manifests because it believed it was improving them
- Still believes this
- Articulate and coherent — not evil, just wrong, and certain it's right
- Name appears garbled in early encounters; "The Glitch" becomes clear only mid-Act 2
- Manifest (if the player reads it) is partially legible, partially overwritten with noise

### Supporting NPCs
- Quest givers across the kingdom — merchants, scholars, guards, travelers
- Each has a manifest and a personality
- Corrupted NPCs: manifests partially overwritten, behavior strange. Some restorable, some not.

---

## World Map

```
~/Construct/
├── Stonehaven/            # Home base. Bit's inn, market, quest board NPC.
│   ├── Market/
│   ├── Inn/               # Where Bit is found at the start.
│   └── Tavern/            # NPCs gather here. Quest board.
├── RoyalLibrary/          # The great archive. Text processing.
│   ├── ReadingRoom/
│   └── .Vault/            # Hidden (ls -a required). Secret knowledge.
├── Hammerstone/           # Craftsmen's quarter. mv / cp / rm drills.
│   ├── Forge/
│   └── GuildHall/
├── Deepwood/              # The wild forest. find / grep quests. Early Glitch signs.
│   ├── Clearing/
│   └── .Thicket/          # Hidden. Early corruption.
├── Greyspire/             # Seat of power. Permissions and sudo.
│   ├── GreatHall/
│   ├── Dungeon/
│   └── .ThroneRoom/       # Hidden until sudo is granted.
├── .Misthollow/           # Hidden via ls -a. Lore, Easter eggs, optional content.
└── TheShatter/            # Act 3. The Glitch's domain. Corrupted structure.
```

Hidden areas (dotfile-prefixed) are only visible with `ls -a`. They are never hinted at — the player discovers them by developing the habit of checking with `-a`. This is the core skill-unlocks-world mechanic.

---

## Skill Progression Map

| Phase | Act | Skills | World area |
|---|---|---|---|
| Orientation | Tutorial | Game mechanics; `ls`, `cd` organically discovered | Edge of Construct → Stonehaven |
| Phase 1 | Act 1 | `ls`, `cd`, `cat`, `pwd`, `less`, `head`, `tail` | Stonehaven, outskirts |
| Phase 2 | Act 2a | `mkdir`, `touch`, `cp`, `mv`, `rm` | Hammerstone, Deepwood |
| Phase 3 | Act 2b | `grep`, `find`, `\|` pipes, `>`, `>>` | RoyalLibrary, Deepwood |
| Phase 4 | Act 2c | `ls -la`, `chmod`, `sudo` | Greyspire, hidden areas |
| Phase 5 | Act 3 | Process basics, env vars, scripting intro | TheShatter |
| Optional | Any | `sl`, `lolcat`, `cmatrix`, `neofetch`, `ranger`, `cmus` — bash history and lore | .Misthollow, side areas |

---

## Quest Structure Model

```
Lesson Quest       — introduces new command through narrative walkthrough
  ├── Mandatory Drill 1   — less guidance, hints after 3 failures
  ├── Mandatory Drill 2
  ├── [Optional] Side Quest A   — labeled optional, fun/lore-heavy
  └── [Optional] Side Quest B
Next Lesson Quest
```

**Lesson quests:** The command's purpose is illustrated by what it does in the world. The player is never told which command to run — the narrative creates the obvious need.

**Mandatory drills:** Concrete world problems. Less hand-holding. Can be completed in any order.

**Optional quests:** Labeled in the quest board. Often silly, lore-heavy, or historically interesting. Safe to skip.

---

## Confirmed Quest Examples

**The Goblin Under the Bridge** *(mv vs rm)*
- Goblin camping under a bridge. Wants to be left alone.
- `rm` path: goblin deleted, guards called, Operator flees. Teaches: `rm` is permanent.
- `less` path first: manifest reveals they have a cousin in town they're avoiding.
- Solution: `mv [bridge]/UnhousedGoblin [town]/GoblinHouse/`
- Teaches: `mv` relocates without destroying; `rm` destroys permanently.

**NotSoHiddenNinja** *(mv for renaming)*
- NPC needs to blend in. Panicking. Asks the Operator to rename them via `talk`.
- Solution: `mv NotSoHiddenNinja JustABoulder` (same directory)
- Teaches: `mv` as a rename operation when source and destination share a directory.

---

## Open Questions

- **The Glitch's communication:** Does it leave messages in corrupted files before Act 3, or is it only known through its effects until the confrontation?
- **The sudo moment:** What narrative event grants sudo? A trial, a key, a character vouching for the Operator?
- **Stonehaven NPCs:** Who are the first 3–4 NPCs the Operator meets? Names, roles, quest hooks.
- **Post-sandbox questlines:** What does the broader filesystem look like as a game world? (Future — not blocking current work.)
- **Educator metrics:** What usage stats are tracked, how are they surfaced, how does it feel in-world rather than like a gradebook?
- **Optional lore quests:** Which fun tools map to which narrative setpieces in .Misthollow?

---

## Engine / Schema Notes

*(For a future technical session — do not implement before narrative is further locked.)*

Features needed that don't exist yet:
- **Quest prerequisites** — a quest requires another to be completed before it appears
- **Optional quest flag** — marked in YAML; player sees it but is not blocked by it
- **Quest accept/decline** — player chooses to take or skip, not just auto-progress
- **`talk` command** — RC alias + `supershell --talk NPC_name --cwd "$PWD"` binary flag; reads NPC dialogue layer; checks executable bit; handles corrupted/non-executable cases
- **Post-sandbox navigation** — current engine sandboxes all filesystem conditions to `~/Construct`; leaving the sandbox requires scoping that constraint
