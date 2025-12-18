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
                if let Ok(content) = fs::read_to_string(path) {
                    let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                    re.is_match(&content)
                } else {
                    false
                }
            }
            Condition::FileMissing { path } => !Path::new(path).exists(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quest {
    pub id: String,
    pub name: String,

    #[serde(default)]
    pub message: String,

    pub conditions: Vec<Condition>,
    pub next_quest_id: String,
}

pub fn load_quests(path: &str) -> HashMap<String, Quest> {
    let mut db = HashMap::new();

    if Path::new(path).exists() {
        let content = fs::read_to_string(path).unwrap_or_default();
        let quests: Vec<Quest> = serde_yml::from_str(&content).unwrap_or_default();

        for q in quests {
            db.insert(q.id.clone(), q);
        }
    } else {
        println!("[!] WARNING: quests.yaml not found at {}", path);
    }
    db
}
