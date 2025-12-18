################################################################################
#                                                                              #
#    S U P E R S H E L L   ::   ARCHITECTURE & DESIGN RECORD (ADR)             #
#                                                                              #
################################################################################
#  STATUS   : Active Development                                               #
#  LANGUAGE : Rust (2024 Edition)                                              #
#  PLATFORM : Bash (Linux/macOS)                                               #
################################################################################


================================================================================
  1. EXECUTIVE SUMMARY
================================================================================
Supershell is a gamified CLI tutor that wraps a standard Bash session. 

It utilizes a "Stateless Engine" architecture: a Rust binary invoked via shell 
hooks checks user actions against a quest database, updates state, and renders 
narrative elements without running a persistent background daemon.


================================================================================
  2. CORE ARCHITECTURAL DECISIONS
================================================================================

[A] THE "STATELESS LAMBDA" PATTERN
--------------------------------------------------------------------------------
DECISION : Logic resides in a single, compiled Rust binary (`supershell`) that 
           starts, executes, and exits immediately (<10ms target).

CONTEXT  : 
  * Why not Python? Python startup latency (100-200ms) creates input lag on 
    every keystroke, ruining the terminal "feel."
  * Benefit: Zero memory footprint when idle. Simplifies deployment. Eliminates 
    "zombie process" risks.

[B] THE "FAIL-OPEN" INTEGRATION STRATEGY
--------------------------------------------------------------------------------
DECISION : The Bash wrapper (`supershell.sh`) treats the binary as an optional 
           enhancement, not a dependency.

CONTEXT  :
  * Priority: Stability. We must never break the user's shell.
  * Mechanism: If the binary crashes, panics, or is missing, the wrapper 
    catches the error and executes the user's command anyway.

[C] THE HOOK IMPLEMENTATION (BASH)
--------------------------------------------------------------------------------
DECISION : Hybrid hook approach due to Bash limitations.

STRATEGY :
  1. Post-Exec (State Update) -> `PROMPT_COMMAND`
     Runs after every interactive command to check if objectives were met.
  
  2. Pre-Exec (Firewall) -> `Alias`
     Example: `alias rm='supershell pre-exec rm'`
     We only "firewall" dangerous commands explicitly. Harmless commands 
     (ls, cat) are only checked after they run.


================================================================================
  3. DATA & STATE MANAGEMENT
================================================================================

[A] ATOMIC STATE PERSISTENCE
--------------------------------------------------------------------------------
DECISION : Save data is written to `save.json.tmp` and atomically renamed to 
           `save.json`.

CONTEXT  : Prevents data corruption if the program crashes midway through a write.

[B] CONCURRENCY CONTROL ("THE GLOBAL LOCK")
--------------------------------------------------------------------------------
DECISION : Binary acquires an exclusive file lock (`supershell.lock`) on startup.
           If locked, the binary exits silently.

CONTEXT  : Prevents race conditions if a user has multiple terminal tabs open.
           If Tab A is playing a cutscene, Tab B simply skips the game check.


================================================================================
  4. UI/UX PHILOSOPHY
================================================================================

[A] THEMATICS OVER LATENCY ("CUTSCENE MODE")
--------------------------------------------------------------------------------
DECISION : On objective completion, the binary enters Raw Mode, clears the 
           screen, and renders text with a typewriter effect.

CONTEXT  : We treat narrative moments as "Cutscenes," intentionally blocking 
           input to force attention to NPC dialogue.
           * Feature: User can press [SPACE] to skip animation.

[B] THE "MISSION STATUS" DASHBOARD
--------------------------------------------------------------------------------
DECISION : Running `supershell` without arguments displays the Quest Log.

CONTEXT  : Replaces standard help text with an immersive status dashboard.


================================================================================
  5. TECHNICAL STACK (RUST 2024)
================================================================================

+----------------+------------------+------------------------------------------+
| COMPONENT      | CRATE / TOOL     | REASON                                   |
+----------------+------------------+------------------------------------------+
| CLI Framework  | clap (v4.5)      | Industry standard argument parsing.      |
+----------------+------------------+------------------------------------------+
| Serialization  | serde_json       | Type-safe parsing of Game State.         |
+----------------+------------------+------------------------------------------+
| Configuration  | serde_yml        | Quest definitions (YAML).                |
+----------------+------------------+------------------------------------------+
| File Locking   | fs2              | Cross-platform locking (Anti-Race).      |
+----------------+------------------+------------------------------------------+
| Regex Engine   | regex            | Validating commands against patterns.    |
+----------------+------------------+------------------------------------------+
| Terminal UI    | crossterm        | Raw-mode control for typewriter effects. |
+----------------+------------------+------------------------------------------+
| Paths          | directories      | Adheres to XDG standards automatically.  |
+----------------+------------------+------------------------------------------+
