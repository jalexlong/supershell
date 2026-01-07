use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Condition {
    CommandMatches { pattern: String },
    FileExists { path: String },
    FileContains { path: String, pattern: String },
    FileMissing { path: String },
}

impl Condition {
    pub fn is_met(&self, user_command: &str) -> bool {
        match self {
            Condition::CommandMatches { pattern } => {
                let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                re.is_match(user_command)
            }
            Condition::FileExists { path } => Path::new(path).exists(),
            Condition::FileContains { path, pattern } => {
                let p = Path::new(path);
                if p.is_file() {
                    if let Ok(content) = fs::read_to_string(p) {
                        let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                        re.is_match(&content)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Condition::FileMissing { path } => !Path::new(path).exists(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Checkpoint {
    pub id: String,
    pub name: String,
    pub instruction: String,
    pub objective: String,
    pub success: String,
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub briefing: String,
    pub debriefing: String,
    pub checkpoints: Vec<Checkpoint>,
    pub next_chapter_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub fn load_chapters(path: &str) -> HashMap<String, Chapter> {
    let mut db = HashMap::new();
    if Path::new(path).exists() {
        let content = fs::read_to_string(path).unwrap_or_default();
        if let Ok(chapters) = serde_yml::from_str::<Vec<Chapter>>(&content) {
            for c in chapters {
                db.insert(c.id.clone(), c);
            }
        }
    }
    db
}
