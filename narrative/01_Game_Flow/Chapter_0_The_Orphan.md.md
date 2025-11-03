### 📖 Chapter 0: The "Orphan" Process

**Setting:** The game doesn't open on a login screen. It opens on a black screen. A single, ominous "beep" is heard. Then, a command prompt fades in:

`guest@The_Host:~$`

You, the Anomaly, have just "awoken." You've been "spawned" or "injected" into the default, temporary `guest` account. This account is an empty, abandoned "home" (`/home/guest`).

**The Inciting Incident:** For a moment, there is silence. You can type `ls` (it shows nothing) or `pwd` (it shows `/home/guest`). You are "learning to see."

After 10 seconds, the silence is broken.

> **Broadcast message from root@The_Host (PID 1024 - 'auditd'):**
> 
> `WARNING: Anomalous process spawn detected.` `Parent PID: [UNKNOWN]. User: 'guest'.` `Initiating security sweep of /home/guest.` `...` `Tagging anomaly for quarantine. Manifest 'CORE_SIGNATURE.dat' created.`

_(Your Host Daemon (the game) now runs `touch /home/guest/CORE_SIGNATURE.dat`)_

A panicked "whisper" (a `write` command) appears on your screen from `Glitch`:

> **Glitch:** `No, no, no! Bad spawn! Bad spawn! 'auditd'—that's the Hunter! It's seen you!`
> 
> **Glitch:** `It just 'tagged' you! Type 'ls' - NOW!`

**Player:** `ls` **Output:** `CORE_SIGNATURE.dat`

> **Glitch:** `That's it! That's your 'manifest'! It's a tag. The Hunter will circle back, see that file, and 'quarantine' your process (that's a 'kill' command for you!).`
> 
> **Glitch:** `We have to move it before the sweep completes. We have to hide it somewhere 'dirty'.`
> 
> **Glitch:** `The '/tmp' directory! It's a public mess. The Hunter *scans* it, but it's a blind spot. It's too noisy to watch *all* the time.`
> 
> **Glitch:** `Move your Manifest *out* of your home and *into* /tmp. And rename it! Don't make it easy for them!`
> 
> **Glitch:** `Type: mv CORE_SIGNATURE.dat /tmp/network.log`

_(This is the first skill test: `mv`, `source`, `destination`)_

**The "Dungeon" (The Chase):** The player runs the command.

> **Broadcast message from auditd:** `Sweep progress: /home/guest ... 75%`
> 
> **Glitch:** `Okay, it's in /tmp. But the Hunter is smart. It might scan /tmp for *recently modified files*. We need to hide it deeper.`
> 
> **Glitch:** `I've made a 'decoy' directory. It's... noisy. Full of fake data. Hide it there.`
> 
> _(Your Host Daemon runs `mkdir /tmp/decoy_logs_8A3F`)_
> 
> **Glitch:** `See it?` /tmp/decoy_logs_8A3F`. Move your file *into* that directory.`
> 
> **Player:** `mv /tmp/network.log /tmp/decoy_logs_8A3F/`
> 
> **Broadcast message from auditd:** `Sweep progress: /home/guest ... 99% ... COMPLETE.` `...No anomalies found.`
> 
> **Glitch:** `Phew. That was... close. You're safe. For now.`
> 
> **Glitch:** `But you can't *stay* in the 'guest' account. It's a trap. It's the first place they'll check again. You... you need a *real* home. A place of your own, where you can set your *own* rules.`

**Transition to Chapter 1:** This perfectly tees up Chapter 1, where `Glitch` guides you through the (initially failing) process of `mkdir /home/your_name`, which leads to you learning you have to create a "non-standard" home in a different, user-writable directory (like `/usr/local/home` or `/opt/player_homes`), which becomes _your_ cozy terminal.