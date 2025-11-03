The Hunter is based on a combination of real Linux security tools: `auditd` and `inotify`.

1. **The "Eye" (`inotify`):** The Host Daemon has a subsystem called `inotify` that "watches" the filesystem. It's a kernel feature that reports events like file creation, deletion, or modification. Your "spawn" as an Anomaly was a _massive_ `inotify` event: **a new, unparented process (you) appeared from nowhere.** This is the "alarm" that woke the Hunter.
    
2. **The "Brain" (`auditd`):** The Hunter process itself is the `auditd` (Audit Daemon). Its job is to _log_ these alarms. It saw the `inotify` event, and its rulebook said: "ALERT: Orphan process spawned in `guest` account. This is a high-priority threat."
    
3. **The "Scan" (The Action):** The Hunter's "scan" is a script that combines:
    
    - `find`: To look for files with suspicious names or recent modification times.
        
    - `grep`: To look for suspicious _content_ inside files.
        
    - `lsof` (List Open Files): To see _who_ is accessing what. This is its secret weapon.
        

**What Traces Does it Leave? (`.trap` files) And Why?** This is the most "gamey" and realistic part. The Hunter _intentionally_ leaves `.trap` files behind. **They are honeypots.**

The Hunter can't scan _every byte_ of the system _all the time_; it's too slow. So, after it sweeps a "noisy" directory like `/tmp`, it _litters_ the area with `.trap` files.

- **Why?** These files are "canaries." The Hunter _knows_ they exist. It puts them on a new "watch list."
    
- **The Trace:** The trace left behind is an `auditd` log. The Hunter's _real_ trap isn't the file; it's the _log entry it creates when you touch the file._
    
- When you run `cat /tmp/1.trap`, the `inotify` system fires, and `auditd` logs: `ALERT: User 'guest' (PID 1234) just read the honeypot file /tmp/1.trap. This user is suspicious. Raising alert level.`
    
- `Glitch` knows this. His advice is: "Don't. Touch. Anything. You don't know what it is."