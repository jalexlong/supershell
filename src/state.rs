use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    pub current_chapter_id: String,
    pub current_checkpoint_index: usize,
    pub is_finished: bool,
}

impl GameState {
    /// Creates a fresh state starting at the beginning of the provided chapter ID
    pub fn new(start_chapter_id: String) -> Self {
        Self {
            current_chapter_id: start_chapter_id,
            current_checkpoint_index: 0,
            is_finished: false,
        }
    }

    /// Loads the save file or initializes a new one if missing
    pub fn load(path: &str, start_chapter_id: String) -> Self {
        if Path::new(path).exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_else(|_| Self::new(start_chapter_id))
        } else {
            Self::new(start_chapter_id)
        }
    }

    pub fn save(&self, path: &str) {
        let tmp_path = format!("{}.tmp", path);
        let json = serde_json::to_string_pretty(self).expect("Failed to serialize");
        fs::write(&tmp_path, json).expect("Failed to write tmp file");
        fs::rename(tmp_path, path).expect("Failed to commit save");
    }

    /// Increments the checkpoint index
    pub fn advance_checkpoint(&mut self) {
        self.current_checkpoint_index += 1;
    }

    /// Moves the user to the start (index 0) of a new chapter
    pub fn move_to_chapter(&mut self, next_chapter_id: String) {
        self.current_chapter_id = next_chapter_id;
        self.current_checkpoint_index = 0;
    }
}
