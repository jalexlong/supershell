# Quest: The Orphan Chase

- **ID:** `00_The_Orphan_Chase`
    
- **Giver:** `The_Host` (via `auditd` event)
    
- **Guide:** `[[Glitch]]`
    
- **Goal:** Survive the initial `auditd` sweep by hiding your `[[CORE_SIGNATURE.dat]]` in `/tmp`.
    
- **Transition:** On success, leads to `[[Chapter_1_Cozy_Home.md]]`.
    

---

### Scene Flow & Script

The scene is tracked by a single variable, e.g., `scene_state`.

#### STATE: `AWAKE`

- **Trigger:** Game start.
    
- **Daemon Action:**
    
    1. Clear screen.
        
    2. Show prompt: `guest@The_Host:~$`
        
    3. Start a 10-second timer.
        
- **Player Action:** Can type commands (`ls`, `pwd`). Daemon should respond with simulated output (e.g., `ls` shows nothing, `pwd` shows `/home/guest`).
    
- **On Timer End:** Transition to `STATE_TAGGED`.
    

---

#### STATE: `TAGGED`

- **Trigger:** `AWAKE` timer ends.
    
- **Daemon Action (Scripted Event):**
    
    1. Print the `Hunter` broadcast.
        
    2. "Spawn" the `CORE_SIGNATURE.dat` file in the game's internal filesystem model (so `ls` will now show it).
        
    3. Print `Glitch`'s first lines.
        
- **Dialogue (Hunter):**
    
    > **Broadcast message from root@The_Host (PID 1024 - 'auditd'):**
    > 
    > `WARNING: Anomalous process spawn detected.` `Parent PID: [UNKNOWN]. User: 'guest'.` `Initiating security sweep of /home/guest.` `...` `Tagging anomaly for quarantine. Manifest 'CORE_SIGNATURE.dat' created.`
    
- **Dialogue (Glitch):**
    
    > **Glitch:** `No, no, no! Bad spawn! Bad spawn! 'auditd'—that's the Hunter! It's seen you!`
    > 
    > **Glitch:** `It just 'tagged' you! Type 'ls' - NOW!`
    
- **Player Action (Listening for `ls`):**
    
    - **If Player types `ls`:**
        
        - **Daemon Output:** `CORE_SIGNATURE.dat`
            
        - **Dialogue (Glitch):**
            
            > **Glitch:** `That's it! That's your 'manifest'! It's a tag. The Hunter will circle back, see that file, and 'quarantine' your process (that's a 'kill' command for you!).`
            > 
            > **Glitch:** `We have to move it before the sweep completes. We have to hide it somewhere 'dirty'.`
            > 
            > **Glitch:** `The '/tmp' directory! It's a public mess. The Hunter *scans* it, but it's a blind spot. It's too noisy to watch *all* the time.`
            > 
            > **Glitch:** `Move your Manifest *out* of your home and *into* /tmp. And rename it! Don't make it easy for them!`
            > 
            > **Glitch:** `Type: mv CORE_SIGNATURE.dat /tmp/network.log`
            
        - **Daemon Action:** Transition to `STATE_HIDING`.
            

---

#### STATE: `HIDING`

- **Trigger:** `Glitch` has given the `mv` instruction.
    
- **Daemon Action:** Listen for player input.
    
    - **If Player types `mv CORE_SIGNATURE.dat /tmp/network.log`:**
        
        1. Internally update the file's location.
            
        2. Transition to `STATE_SAFE`.
            
    - **If Player types anything else (`ls`, `pwd`, mistyped command):**
        
        - **Dialogue (Hunter):** `Sweep progress: /home/guest ... 75%`
            
        - **Dialogue (Glitch):** `That's not it! Hurry! Type: mv CORE_SIGNATURE.dat /tmp/network.log`
            

---

#### STATE: `SAFE`

- **Trigger:** The correct `mv` command is entered.
    
- **Daemon Action:**
    
    1. Simulate "spawning" the decoy directory: `mkdir /tmp/decoy_logs_8A3F`
        
    2. Print the `Hunter` and `Glitch` dialogue for the next step.
        
    3. Transition to `STATE_HIDING_DEEPER`.
        
- **Dialogue (Hunter):** `Sweep progress: /home/guest ... 75%`
    
- **Dialogue (Glitch):**
    
    > **Glitch:** `Okay, it's in /tmp. But the Hunter is smart. It might scan /tmp for *recently modified files*. We need to hide it deeper.`
    > 
    > **Glitch:** `I've made a 'decoy' directory. It's... noisy. Full of fake data. Hide it there.`
    > 
    > **Glitch:** `See it?` /tmp/decoy_logs_8A3F`. Move your file *into* that directory.`
    > 
    > **Glitch:** `Type: mv /tmp/network.log /tmp/decoy_logs_8A3F/`
    

---

#### STATE: `HIDING_DEEPER`

- **Trigger:** `Glitch` has given the second `mv` instruction.
    
- **Daemon Action:** Listen for player input.
    
    - **If Player types `mv /tmp/network.log /tmp/decoy_logs_8A3F/`:**
        
        1. Internally update the file's location.
            
        2. Transition to `STATE_CLEAR`.
            
    - **If Player types anything else:**
        
        - **Dialogue (Hunter):** `Sweep progress: /home/guest ... 90%`
            
        - **Dialogue (Glitch):** `Not quite! Be careful with the path! Try again!`
            

---

#### STATE: `CLEAR`

- **Trigger:** The correct second `mv` command is entered.
    
- **Daemon Action:**
    
    1. Print the final `Hunter` broadcast.
        
    2. Print `Glitch`'s concluding dialogue, which bridges to Chapter 1.
        
- **Dialogue (Hunter):**
    
    > **Broadcast message from auditd:** `Sweep progress: /home/guest ... 99% ... COMPLETE.` `...No anomalies found.`
    
- **Dialogue (Glitch):**
    
    > **Glitch:** `Phew. That was... close. You're safe. For now.`
    > 
    > **Glitch:** `But you can't *stay* in the 'guest' account. It's a trap. It's the first place they'll check again. You... you need a *real* home. A place of your own, where you can set your *own* rules.`
    > 
    > **Glitch:** `Let's get you set up. This is where the real fun begins...`
    
- **End Scene:**
    
    - **Daemon Action:** Load [[Chapter_1_Home_Sweet_Home.md]] logic.