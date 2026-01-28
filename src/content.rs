// content.rs

use crate::state::GameState;
use log::{debug, error, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yml;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// --- DATA STRUCTURES (The "Shape" of our Game) ---

/// Represents a single task the user must complete.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Objective {
    #[serde(rename = "objective")]
    pub title: String,
    pub description: String,
    pub success_msg: String,
    pub conditions: Vec<Condition>,
}

/// A wrapper around our ConditionType enum.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Condition {
    #[serde(flatten)]
    pub condition_type: ConditionType,
    #[serde(default)]
    pub negate: bool,
}

/// The specific types of checks we can perform.
/// Tagged with "type" so the YAML knows which one is which.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ConditionType {
    CommandMatches { pattern: String },
    WorkingDir { path: String },
    FileExists { path: String },
    FileContentMatches { path: String, pattern: String },
}

/// A simple enum to communicate success/failure back to main.rs
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl Condition {
    pub fn check(&self, user_cmd: &str, _game: &GameState) -> ValidationResult {
        // 1. Core Logic
        let result = match &self.condition_type {
            ConditionType::CommandMatches { pattern } => {
                let re = Regex::new(pattern).unwrap();
                let matched = re.is_match(user_cmd);
                debug!(
                    "Condition [CommandMatches]: Input='{}' Pattern='{}' Match={}",
                    user_cmd, pattern, matched
                );
                matched
            }
            ConditionType::WorkingDir { path } => {
                let current = env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let re = Regex::new(path).unwrap();
                let matched = re.is_match(&current);
                debug!(
                    "Condition [WorkingDir]: PWD='{}' Target='{}' Match={}",
                    current, path, matched
                );
                matched
            }
            ConditionType::FileExists { path } => {
                let exists = Path::new(path).exists();
                debug!("Condition [FileExists]: Path='{}' Exists={}", path, exists);
                exists
            }
            ConditionType::FileContentMatches { path, pattern } => {
                if let Ok(content) = fs::read_to_string(path) {
                    let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new(".*").unwrap());
                    // NOTE: Add debug!() call here later
                    re.is_match(&content)
                } else {
                    false
                }
            }
        };

        // 2. Negation Logic
        let final_result = if self.negate { !result } else { result };

        if final_result {
            ValidationResult::Valid
        } else {
            ValidationResult::Invalid("Condition not met".into())
        }
    }
}

// --- LIBRARY SYSTEM ---

pub struct Library {
    pub root_dir: PathBuf,
}

impl Library {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    /// Scans the library folder for .yaml files (Quests)
    pub fn list_modules(&self) -> Vec<(PathBuf, String)> {
        let mut modules = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.root_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        match Module::load(&path) {
                            Ok(module) => {
                                let display_name = if module.title.is_empty() {
                                    path.file_stem().unwrap().to_string_lossy().to_string()
                                } else {
                                    module.title
                                };
                                modules.push((path, display_name));
                            }
                            Err(e) => {
                                // Important: Log corrupt files so we know to fix them
                                warn!("Failed to load module {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }
        // Alphabetize the list
        modules.sort_by(|a, b| a.1.cmp(&b.1));
        modules
    }

    /// Finds a specific course by name
    pub fn get_module(&self, module_name: &str) -> Option<Module> {
        // 1. Try finding it directly by filename
        let path = self.root_dir.join(format!("{}.yaml", module_name));
        if path.exists() {
            return Module::load(&path).ok();
        }

        // 2. Fallback: Search inside files (slower but safer)
        self.list_modules()
            .into_iter()
            .find(|(p, _)| p.file_stem().unwrap().to_string_lossy() == module_name)
            .and_then(|(p, _)| Module::load(&p).ok())
    }
}

/// Represents an entire Level (Course)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Module {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub intro: String,
    #[serde(default)]
    pub outro: String,
    pub missions: Vec<Mission>,
    #[serde(default)] // If missing in YAML, use empty list
    pub setup_actions: Vec<crate::actions::SetupAction>,
}

impl Module {
    /// Helper to read a file and parse YAML into a Struct
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        match serde_yml::from_str(&content) {
            Ok(c) => Ok(c),
            Err(e) => {
                error!("YAML Error in {:?}: {}", path, e);
                Err(Box::new(e))
            }
        }
    }
}

/// Represents a Chapter (Level) within a Course
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mission {
    pub title: String,
    pub intro: String,
    pub outro: String,
    pub objectives: Vec<Objective>,
    #[serde(default)]
    pub setup_actions: Vec<crate::actions::SetupAction>,
}
