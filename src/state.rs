use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    #[serde(default)]
    pub current_course: String,
    #[serde(default)]
    pub course_version: String,
    pub current_quest_id: String,
    pub current_chapter_index: usize,
    pub current_task_index: usize,
    #[serde(default)]
    pub flags: HashMap<String, bool>,
    #[serde(default)]
    pub variables: HashMap<String, i32>,
    pub is_finished: bool,
}

impl GameState {
    /// Initialize defaults.
    pub fn new() -> Self {
        Self {
            current_course: String::new(),
            course_version: String::new(),
            current_quest_id: String::new(),
            current_chapter_index: 0,
            current_task_index: 0,
            flags: HashMap::new(),
            variables: HashMap::new(),
            is_finished: false,
        }
    }

    /// Loads the save file or initializes a new one if missing.
    /// Warns and backs up a corrupted save rather than silently discarding it.
    pub fn load(path: &str) -> Self {
        if !Path::new(path).exists() {
            return Self::new();
        }
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(">> [WARN] Could not read save file: {e}. Starting fresh.");
                return Self::new();
            }
        };
        match serde_json::from_str::<Self>(&content) {
            Ok(state) => state,
            Err(e) => {
                let backup = format!("{}.bak", path);
                eprintln!(
                    ">> [WARN] Save file corrupted ({e}). Backing up to {backup} and starting fresh."
                );
                let _ = fs::rename(path, &backup);
                Self::new()
            }
        }
    }

    pub fn save(&self, path: &str) -> std::io::Result<()> {
        // 1. Ensure the save directory exists.
        // This matters after --reset, in fresh installs, and in test environments.
        if let Some(parent) = std::path::Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }

        // 2. Create a temporary path.
        let tmp_path = format!("{}.tmp", path);

        // 3. Serialize to string.
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;

        // 4. Write to the temporary file.
        // If we crash here, only the .tmp file is broken.
        fs::write(&tmp_path, json)?;

        // 5. Rename into place.
        // On POSIX systems, this operation is atomic.
        fs::rename(tmp_path, path)?;

        Ok(())
    }

    /// Increments the checkpoint index
    pub fn advance_task(&mut self) {
        self.current_task_index += 1;
    }

    /// Moves the user to the start (index 0) of a new chapter
    pub fn advance_chapter(&mut self) {
        self.current_chapter_index += 1;
        self.current_task_index = 0;
    }

    /// Set a boolean flag (e.g., "tutorial_complete" -> true)
    pub fn set_flag(&mut self, key: &str, value: bool) {
        self.flags.insert(key.to_string(), value);
    }

    /// Check a flag (defaults ot false if not set)
    pub fn get_flag(&self, key: &str) -> bool {
        *self.flags.get(key).unwrap_or(&false)
    }

    /// Set an integer variable (e.g., "credits" -> 50)
    pub fn set_var(&mut self, key: &str, value: i32) {
        self.variables.insert(key.to_string(), value);
    }

    /// Get a variable (defaults to 0 if not set)
    pub fn get_var(&self, key: &str) -> i32 {
        *self.variables.get(key).unwrap_or(&0)
    }

    /// Modify a variable (e.g., add 10 points)
    pub fn mod_var(&mut self, key: &str, amount: i32) {
        let current = self.get_var(key);
        self.variables.insert(key.to_string(), current + amount);
    }
}
