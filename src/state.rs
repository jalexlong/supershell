use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    #[serde(default)]
    pub current_module: String,
    #[serde(default)]
    pub module_version: String,
    pub current_mission_index: usize,
    pub current_objective_index: usize,

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
            current_module: String::new(),
            module_version: String::new(),
            current_mission_index: 0,
            current_objective_index: 0,
            flags: HashMap::new(),
            variables: HashMap::new(),
            is_finished: false,
        }
    }

    /// Loads the save file or initializes a new one if missing
    pub fn load(path: &str) -> Self {
        if Path::new(path).exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }

    pub fn save(&self, path: &str) {
        let tmp_path = format!("{}.tmp", path);
        let json = serde_json::to_string_pretty(self).expect("Failed to serialize");
        fs::write(&tmp_path, json).expect("Failed to write tmp file");
        fs::rename(tmp_path, path).expect("Failed to commit save");
    }

    /// Increments the checkpoint index
    pub fn advance_objective(&mut self) {
        self.current_objective_index += 1;
    }

    /// Moves the user to the start (index 0) of a new chapter
    pub fn advance_mission(&mut self) {
        self.current_mission_index += 1;
        self.current_objective_index = 0;
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
