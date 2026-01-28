// quest.rs

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
pub struct Task {
    pub objective: String,
    pub description: String,
    pub success_msg: String,
    pub conditions: Vec<Condition>,
}

/// A wrapper around our ConditionType enum.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Condition {
    #[serde(flatten)]
    pub condition_type: ConditionType,
}

/// The specific types of checks we can perform.
/// Tagged with "type" so the YAML knows which one is which.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ConditionType {
    CommandMatches { pattern: String },
    WorkingDir { path: String },
    FileExists { path: String },
}

/// A simple enum to communicate success/failure back to main.rs
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl Condition {
    /// The "Logic Engine". Takes user input + game state and returns Pass/Fail.
    pub fn check(&self, user_cmd: &str, _game: &GameState) -> ValidationResult {
        let is_valid = match &self.condition_type {
            // LOGIC: Check if the user typed the correct command
            ConditionType::CommandMatches { pattern } => {
                let re = Regex::new(pattern).unwrap();
                let matched = re.is_match(user_cmd);
                debug!(
                    "Condition [CommandMatches]: Input='{}' Pattern='{}' Match={}",
                    user_cmd, pattern, matched
                );
                matched
            }

            // LOGIC: Check if the user is in the correct folder
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

            // LOGIC: Check if a specific file exists
            ConditionType::FileExists { path } => {
                let exists = Path::new(path).exists();
                debug!("Condition [FileExists]: Path='{}' Exists={}", path, exists);
                exists
            }
        };

        if is_valid {
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
    pub fn list_available_courses(&self) -> Vec<(PathBuf, String)> {
        let mut courses = Vec::new();

        // Safety check: Does directory exist?
        if let Ok(entries) = fs::read_dir(&self.root_dir) {
            // Flatten removes Err results, giving us only valid entries
            for entry in entries.flatten() {
                let path = entry.path();
                // We only care about .yaml files
                if let Some(ext) = path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        // Try to load the course to get its title
                        match Course::load(&path) {
                            Ok(course) => {
                                let display_name = if course.title.is_empty() {
                                    path.file_stem().unwrap().to_string_lossy().to_string()
                                } else {
                                    course.title
                                };
                                courses.push((path, display_name));
                            }
                            Err(e) => {
                                // Important: Log corrupt files so we know to fix them
                                warn!("Failed to load course file {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }
        // Alphabetize the list
        courses.sort_by(|a, b| a.1.cmp(&b.1));
        courses
    }

    /// Finds a specific course by name
    pub fn get_course(&self, course_name: &str) -> Option<Course> {
        // 1. Try finding it directly by filename
        let path = self.root_dir.join(format!("{}.yaml", course_name));
        if path.exists() {
            return Course::load(&path).ok();
        }

        // 2. Fallback: Search inside files (slower but safer)
        self.list_available_courses()
            .into_iter()
            .find(|(p, _)| p.file_stem().unwrap().to_string_lossy() == course_name)
            .and_then(|(p, _)| Course::load(&p).ok())
    }
}

/// Represents an entire Level (Course)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Course {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub intro: String,
    #[serde(default)]
    pub outro: String,
    pub chapters: Vec<Chapter>,
    #[serde(default)] // If missing in YAML, use empty list
    pub setup_actions: Vec<crate::actions::SetupAction>,
}

impl Course {
    /// Helper to read a file and parse YAML into a Struct
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        match serde_yml::from_str(&content) {
            Ok(c) => Ok(c),
            Err(e) => {
                error!("YAML Deserialization Error in {:?}: {}", path, e);
                Err(Box::new(e))
            }
        }
    }
}

/// Represents a Chapter (Level) within a Course
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub title: String,
    pub intro: String,
    pub outro: String,
    pub tasks: Vec<Task>,
    // Crucial for "World Engine" to know what folders to create!
    #[serde(default)]
    pub setup_actions: Vec<crate::actions::SetupAction>,
}
