# Supershell: Game Design Document

## 1. High Concept
* **The Pitch:** A silent, narrative layer that sits on top of your real terminal. It transforms a standard learning session into a Cyberpunk mystery without taking over the screen.
* **Target Audience:** Absolute beginners to CLI, CS students, and developers wanting to sharpen regex/piping skills.
* **Core Loop:**
    1.  **Briefing:** User sees a minimal, inline mission card (e.g., "Objective: Locate the inhibitor.").
    2.  **Action:** User performs *real* shell tasks (e.g., `ls`, `rm memory_block.dat`).
    3.  **Validation:** The shell session uses a `_g` alias interceptor (transparent to the player — identical UX to implicit hooking). Every relevant command is piped through `supershell --check` automatically.
    4.  **Reaction:**
        * *Success:* Instant Green Status Card with the next objective.
        * *Failure:* A "Glitch" effect triggers: the error message is rendered with Unicode combining strikethrough (U+0336), giving a text-corruption visual. After three consecutive failures the player receives a hint.
        * *Neutral:* If the command is valid but unrelated (e.g., `whoami`), the game remains silent.

## 2. Gameplay Mechanics

### 2.1 Knowledge-Gated Progression
* **No Unlocks:** The player starts with full root permissions (conceptually) and all tools (`grep`, `awk`, `ssh`) available.
* **The Challenge:** The "game" is the player realizing they *need* a specific tool. The narrative provides the *why*, the player must deduce the *how*.
* **Hint System:** If the player fails the same task three times in a row, the game injects the task's `hint` field into the failure output as a "Corrupted Data Fragment". The failure count persists across restarts; it resets when the task is completed. *(Implemented: M5)*

### 2.2 Failure State: "The Glitch"
* **No Health Bars:** The user cannot "die."
* **Visual Feedback:** Failure is indicated by text corruption — the error message is rendered with Unicode strikethrough combining characters, producing the `E̶R̶R̶O̶R̶:̶ ̶A̶C̶C̶E̶S̶S̶ ̶D̶E̶N̶I̶E̶D̶` effect from the GDD. In test mode the combining characters are suppressed to keep test assertions readable. *(Implemented: M6)*
* **World Reset:** If the user destroys the environment (e.g., `rm -rf ~/Construct`), the World Engine detects the missing root on the next command check and performs an instant "System Restore," re-creating the directory and re-running the current chapter's setup actions. *(Implemented: M6)*

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
    2.  **Navigate:** User types `cd Memory_Bank` to enter the memory store.
    3.  **Search:** User types `ls` inside the Memory Bank.
    4.  **Read:** User types `cat welcome_packet.txt` to receive their first instruction.

*(The original GDD listed `rm inhibitor.dat` and `mkdir escape_tunnel` as objectives. These were replaced in the current intro module with navigation-focused tasks that better serve absolute beginners. They remain as candidates for a future "Advanced Awakening" quest.)*

## 4. UI/UX Constraints
* **No HUD:** The player relies on their standard terminal prompt.
* **Notifications:** Only inline text inside the standard scrollback. Clearing the screen is used only at chapter transitions.
* **Visual Style:**
    * **Success:** Clean, White/Green, Box Drawing Characters.
    * **Glitch:** Red, Unicode combining strikethrough on the failure message.
    * **Hint:** Yellow, surfaced after three consecutive failures on the same task.
