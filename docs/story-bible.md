# Supershell Story Bible

## Working Theme

**The Derelict Data Ark**

Supershell takes place aboard a damaged instructional vessel known as the **Ark**. The Ark was built to preserve knowledge, train Operators, and maintain secure systems after a catastrophic network collapse.

The Ark is still operational, but many of its training systems are degraded, fragmented, or offline.

The player is not an attacker. The player is an **Operator-in-training** learning to navigate, inspect, repair, and secure the Ark using real shell commands.

## Core Design Pillar

```text
real shell behavior first, game overlay second
```

The fiction should reinforce real terminal concepts instead of replacing them.

Commands are not magic spells. They are real shell commands interpreted through the Ark's training interface.

## Core World Metaphor

Supershell's story should translate shell concepts into simple adventure-game concepts:

```text
Directories are places.
Files are objects, items, or things.
Commands are actions.
```

This metaphor should guide the writing.

A directory is not just a technical container. In the player's imagination, it is a place they can enter.

A file is not just data. It is an object they can inspect, read, move, copy, or eventually modify.

A command is not a spell or fake game action. It is a real shell action that changes what the player sees or does.

## Player Role

The player is an **Operator Candidate** awakened inside a damaged training deck.

The Operator's job is to:

- navigate the Ark
- inspect unfamiliar places
- find and examine objects
- read system logs
- recover knowledge packets
- repair broken training modules
- understand access boundaries
- secure systems responsibly

The player should feel capable, curious, and responsible.

## World Premise

The Ark is a derelict learning vessel.

Its decks contain places, objects, systems, and traces of previous Operators.

The Ark is not evil. It is damaged.

The player is learning to become useful inside a real system.

The world should feel like a mystery game. The player should have to observe carefully, make connections, and think through problems, but the game should provide enough training wheels that beginners do not feel abandoned.

## Story Tone

The tone should be:

- mysterious but not confusing
- serious but not grim
- technical but beginner-friendly
- adventurous but safety-minded
- thoughtful without being slow
- encouraging without being childish

The tone should not be:

- goofy
- edgy
- grimdark
- an "elite hacker simulator"
- full of fake hacking clichés

Supershell should feel like a mystery game about learning how systems work.

The player should feel like they are exploring, investigating, repairing, and slowly understanding a strange environment.

Avoid making the player feel like they are "hacking everything." Supershell should teach careful system operation, not reckless intrusion.

## Core Motifs

Use these often:

- decks
- rooms
- compartments
- corridors
- bulkheads
- relays
- manifests
- diagnostic logs
- access panels
- training modules
- signal traces
- maintenance nodes
- archive packets
- emergency lights
- damaged systems
- safe operating procedures
- strange clues
- locked paths
- hidden meanings
- recovered messages

## Preferred Vocabulary

Use these words frequently:

- Operator
- Ark
- deck
- room
- place
- compartment
- object
- item
- manifest
- diagnostic
- relay
- uplink
- access
- repair
- inspect
- trace
- recover
- secure
- archive
- module
- protocol
- clue
- signal
- log

## Vocabulary to Use Carefully

These words are allowed later, especially in cybersecurity modules, but should not dominate the intro:

- hack
- exploit
- breach
- attack
- weapon
- malware
- virus
- target

Early Supershell content should build responsible mental models before introducing adversarial security concepts.

## Guide Character

The guide is the **Archivist**, a damaged but functional training intelligence aboard the Ark.

The Archivist should:

- explain tasks clearly
- stay calm when the player makes mistakes
- avoid sarcasm
- encourage careful observation
- translate shell actions into Ark-world meaning
- remind the player that real systems require precision
- give hints without solving every problem immediately

Example voice:

> Operator, your console is active. This place is dark because no scan has been requested. Use `ls` to list what is visible here.

## Shell-to-World Mapping

| Shell Concept | Ark Metaphor |
|---|---|
| current directory | current place |
| directory | place, room, deck, compartment, corridor, or system area |
| file | object, item, log, packet, manifest, note, artifact, or clue |
| command | action |
| `pwd` | check your current location |
| `ls` | scan the current place |
| `cd` | move to another place |
| `cat` | inspect, read, or decode an object |
| `mkdir` | create a new place |
| `touch` | create a marker, note, or blank object |
| `cp` | duplicate an object |
| `mv` | move or rename an object |
| `rm` | delete an object; must be treated as risky |
| permissions | access control |
| pipe | route a signal |
| redirect | capture or send output |
| process | running subsystem |
| network | remote relay or uplink |

## Mystery Design Principles

Supershell should challenge students to think outside the box, but not leave them stranded.

Good mystery design for Supershell means:

- the player always has a clear immediate goal
- the player may need to inspect the environment to know what to do next
- clues should be discoverable through real commands
- hints should teach shell reasoning, not just give answers
- failure should be recoverable
- early puzzles should have strong training wheels
- later puzzles can remove some guidance gradually

A good Supershell puzzle should make the student think:

> "What do I know? Where am I? What can I inspect? What action makes sense?"

It should not make the student think:

> "What random command does the game want?"

## Intro Module Direction

The first module should teach navigation and reading without adding unnecessary mechanics.

Current command sequence:

```bash
ls
cd Memory_Bank/
ls
cd Sector_A/
cat welcome_packet.txt
```

New story wrapper:

1. **Cold Start**
   - Teach `ls`.
   - The Operator wakes in a dark training area.
   - `ls` scans the current place.

2. **First Passage**
   - Teach `cd Memory_Bank/`.
   - Directories are places the Operator can enter.

3. **Survey the Room**
   - Teach `ls` after moving.
   - Moving does not automatically reveal everything in the new place.

4. **Lower Deck Access**
   - Teach `cd Sector_A/`.
   - Places can contain other places.

5. **Read the Object**
   - Teach `cat welcome_packet.txt`.
   - Files are objects that can be inspected or read.

## Intro Learning Goals

By the end of the intro, the player should understand:

- a shell prompt represents a location
- `ls` lists what is visible in the current location
- `cd` changes location
- directories are places
- directories can contain other directories
- files are objects, items, or things
- files are different from directories
- `cat` prints the contents of a readable file
- real commands drive progress

## Safety and Classroom Framing

Supershell should repeatedly model safe habits:

- observe before acting
- read messages carefully
- avoid destructive commands until taught
- understand where you are before changing things
- treat systems as environments to maintain, not toys to break
- respect access boundaries
- think before running commands that modify or delete things

## Content Rules

Quest content should:

- teach one new concept at a time
- use forgiving but precise command patterns
- give useful success messages
- avoid long lore dumps before commands
- make failure recoverable
- keep shell behavior real
- reward observation
- avoid fake commands
- avoid pretending the shell works differently than it really does

## Naming Guidelines

Recommended early places:

- `Training_Deck`
- `Memory_Bank`
- `Sector_A`
- `Archive_Core`
- `Relay_Room`
- `Maintenance_Bay`
- `Signal_Bridge`
- `Observation_Post`
- `Diagnostics_Bay`

Recommended early objects:

- `welcome_packet.txt`
- `deck_manifest.txt`
- `operator_notes.txt`
- `diagnostic_log.txt`
- `access_briefing.txt`
- `signal_trace.txt`
- `maintenance_record.txt`

## Future Module Ideas

### Module 1: Reactivation

Basic navigation and file reading.

Commands:

```bash
ls
cd
cat
pwd
```

### Module 2: Maintenance Deck

Creating and organizing files.

Commands:

```bash
mkdir
touch
cp
mv
```

### Module 3: Archive Recovery

Searching text and reading logs.

Commands:

```bash
grep
head
tail
less
wc
```

### Module 4: Access Control

Permissions and ownership concepts.

Commands:

```bash
chmod
ls -l
```

### Module 5: Signal Routing

Pipes and redirects.

Commands:

```bash
|
>
>>
```

### Module 6: Remote Relay

Intro networking and responsible remote access.

Commands:

```bash
ping
ssh
scp
```

## Current Canon

Supershell is about becoming a skilled Operator.

The Ark is a damaged training environment, not an enemy.

The player succeeds by observing, understanding, and repairing systems using real shell commands.

The world should feel mysterious, but the learning path should be fair.

Directories are places.

Files are objects, items, or things.

Commands are actions.
