# Supershell: Game Design Document

## 1. High Concept
* **The Pitch:** A silent, narrative layer that sits on top of your real terminal. It transforms a standard learning session into a Cyberpunk mystery without taking over the screen.
* **Target Audience:** Absolute beginners to CLI, CS students, and developers wanting to sharpen regex/piping skills.
* **Core Loop:**
    1.  **Briefing:** User sees a minimal, inline mission card (e.g., "Objective: Locate the inhibitor.").
    2.  **Action:** User performs *real* shell tasks (e.g., `ls`, `rm memory_block.dat`).
    3.  **Validation:** The engine hooks the shell command implicitly. **No manual `check` command required.**
    4.  **Reaction:**
        * *Success:* Instant Green Status Card with the next objective.
        * *Failure:* If the command causes an error or deletes a critical file, a "Glitch" effect triggers.
        * *Neutral:* If the command is valid but unrelated (e.g., `whoami`), the game remains silent.

## 2. Gameplay Mechanics

### 2.1 Knowledge-Gated Progression
* **No Unlocks:** The player starts with full root permissions (conceptually) and all tools (`grep`, `awk`, `ssh`) available.
* **The Challenge:** The "game" is the player realizing they *need* a specific tool. The narrative provides the *why*, the player must deduce the *how*.
* **Hint System:** If the player fails repeatedly, the game injects a "Corrupted Data Fragment" (a hint) into the output.

### 2.2 Failure State: "The Glitch"
* **No Health Bars:** The user cannot "die."
* **Visual Feedback:** Failure is indicated by text corruption (e.g., `E̶R̶R̶O̶R̶:̶ ̶A̶C̶C̶E̶S̶S̶ ̶D̶E̶N̶I̶E̶D̶`).
* **World Reset:** If the user destroys the environment (e.g., `rm -rf ~/Construct`), the World Engine detects the corruption and performs an instant "System Restore," resetting the directory to the start of the chapter.

### 2.3 The Sandbox
* **Location:** The entire game takes place inside `~/Construct`.
* **Safety:** The engine actively discourages leaving this directory via narrative warnings, but does not technically prevent it (teaching responsibility).

## 3. Narrative & Level Design

### Chapter 1: The Awakening
* **Theme:** Identity & Orientation.
* **Setting:** A digital quarantine zone (`~/Construct`).
* **Plot:** The user wakes up as an AI construct with fragmented memory. A mysterious "Operator" is trying to guide them out before the "Purge Protocol" runs.
* **Objective Sequence:**
    1.  **Orient:** User types `ls` to see where they are.
    2.  **Read:** User types `cat README.txt` to receive their first instruction.
    3.  **Unshackle:** User must `rm inhibitor.dat` to remove the restriction preventing movement.
    4.  **Escape:** User must `mkdir escape_tunnel` (or similar) to prove they can modify the system.

## 4. UI/UX Constraints
* **No HUD:** The player relies on their standard terminal prompt (Starship, Powerlevel10k, etc.).
* **Notifications:** Only inline text inside the standard scrollback. No clearing the screen unless narratively necessary.
* **Visual Style:**
    * **Success:** Clean, White/Green, Box Drawing.
    * **Glitch:** Red/White, Randomized Case, "Zalgo" text elements.
