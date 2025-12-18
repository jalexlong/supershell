use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameState {
    // The "Bookmark": Which story beat is active?
    pub current_quest_id: String,

    // The "Journal": A list of everything the user has finished.
    pub completed_quests: HashSet<String>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            current_quest_id: "00_init".to_string(),
            completed_quests: HashSet::new(),
        }
    }

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

    pub fn complete_current_quest(&mut self, next_quest_id: &str) {
        self.completed_quests.insert(self.current_quest_id.clone());
        self.current_quest_id = next_quest_id.to_string();
    }
}
