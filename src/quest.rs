use directories::UserDirs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Condition {
    // INPUT CHECKS
    CommandMatches { pattern: String },
    HistoryContains { pattern: String },

    // EXISTENCE CHECKS
    PathExists { path: String },
    PathMissing { path: String },

    // TYPE CHECKS
    IsDirectory { path: String },
    IsFile { path: String },
    IsExecutable { path: String },

    // LOCATION CHECKS
    WorkingDir { pattern: String },

    // CONTENT CHECKS
    FileContains { path: String, pattern: String },
    FileNotContains { path: String, pattern: String },
    FileEmpty { path: String },

    // ENVIRONMENT CHECKS
    EnvVar { name: String, value: String },
}

impl Condition {
    pub fn is_met(&self, user_command: &str) -> bool {
        match self {
            Condition::CommandMatches { pattern } => {
                let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                re.is_match(user_command)
            }
            Condition::HistoryContains { pattern } => {
                if let Some(user_dirs) = UserDirs::new() {
                    let history_path = user_dirs.home_dir().join(".bash_history");

                    if let Ok(content) = fs::read_to_string(history_path) {
                        let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                        re.is_match(&content)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Condition::PathExists { path } => Path::new(path).exists(),
            Condition::PathMissing { path } => !Path::new(path).exists(),
            Condition::IsDirectory { path } => Path::new(path).is_dir(),
            Condition::IsFile { path } => Path::new(path).is_file(),
            Condition::IsExecutable { path } => {
                if let Ok(metadata) = fs::metadata(path) {
                    // Check if the executable bit (0o111) is set
                    metadata.permissions().mode() & 0o111 != 0
                } else {
                    false
                }
            }
            Condition::WorkingDir { pattern } => {
                let current_dir = env::current_dir().unwrap_or_default();
                let path_str = current_dir.to_string_lossy();
                let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                re.is_match(&path_str)
            }
            Condition::FileContains { path, pattern } => {
                if let Ok(content) = fs::read_to_string(path) {
                    let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                    re.is_match(&content)
                } else {
                    false
                }
            }
            Condition::FileNotContains { path, pattern } => {
                if let Ok(content) = fs::read_to_string(path) {
                    let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                    !re.is_match(&content)
                } else {
                    true
                }
            }
            Condition::FileEmpty { path } => {
                if let Ok(metadata) = fs::metadata(path) {
                    metadata.len() == 0
                } else {
                    false
                }
            }
            Condition::EnvVar { name, value } => match env::var(name) {
                Ok(val) => val == *value,
                Err(_) => false,
            },
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
