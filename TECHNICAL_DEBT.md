## Current Architecture
- **Language:** Rust (v1.90+)
- **State Storage:** JSON (standardized via `directories-next` crate)
- **Data Format:** YAML (parsed via `serde_yml`)
- **UI Logic:** Blocking TTY reads for "Press Space to Continue" interactions.

## Known Constraints
- The shell hook currently relies on `history 1`, which may vary slightly between Bash and Zsh configurations.
- Progress is strictly linear (Branching/Choice logic was removed for simplicity in v0.2.0).
