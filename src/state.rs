use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    #[serde(default)]
    pub current_course: String,
    pub current_quest_id: String,
    pub current_chapter_index: usize,
    pub current_task_index: usize,
    pub is_finished: bool,
}

impl GameState {
    /// Initialize defaults.
    pub fn new() -> Self {
        Self {
            current_course: String::new(),
            current_quest_id: String::new(),
            current_chapter_index: 0,
            current_task_index: 0,
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
        // Atomic save: Write to tmp then rename to prevent potential data loss
        let tmp_path = format!("{}.tmp", path);
        let json = serde_json::to_string_pretty(self).expect("Failed to serialize");
        fs::write(&tmp_path, json).expect("Failed to write tmp file");
        fs::rename(tmp_path, path).expect("Failed to commit save");
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
}
