use crate::actions::SetupAction;
use crate::state::GameState;
use directories::UserDirs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// --- LIBRARY & COURSE STRUCTS ---
pub struct Library {
    pub root_dir: PathBuf,
}

impl Library {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn list_available_courses(&self) -> Vec<(PathBuf, String)> {
        let mut courses = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.root_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        let course = Course::load(&path);
                        let display_name = if course.title == "Untitled Course" {
                            path.file_stem().unwrap().to_string_lossy().to_string()
                        } else {
                            course.title
                        };
                        courses.push((path, display_name));
                    }
                }
            }
        }
        courses.sort_by(|a, b| a.1.cmp(&b.1));
        courses
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Course {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub quests: Vec<Quest>,
}

fn default_title() -> String {
    "Untitled Course".to_string()
}
fn default_author() -> String {
    "Anonymous".to_string()
}
fn default_version() -> String {
    "0.0.0".to_string()
}

impl Course {
    pub fn load(path: &Path) -> Self {
        if !path.exists() {
            return Course {
                title: default_title(),
                author: default_author(),
                version: default_version(),
                quests: vec![],
            };
        }
        let content = fs::read_to_string(path).unwrap_or_default();
        match serde_yml::from_str::<Course>(&content) {
            Ok(course) => course,
            Err(_) => match serde_yml::from_str::<Vec<Quest>>(&content) {
                Ok(quests) => Course {
                    title: default_title(),
                    author: default_author(),
                    version: default_version(),
                    quests,
                },
                Err(_) => match serde_yml::from_str::<Quest>(&content) {
                    Ok(quest) => Course {
                        title: default_title(),
                        author: default_author(),
                        version: default_version(),
                        quests: vec![quest],
                    },
                    Err(e) => panic!("YAML Error: {}", e),
                },
            },
        }
    }

    pub fn get_active_content(
        &self,
        quest_id: &str,
        chapter_idx: usize,
        task_idx: usize,
    ) -> Option<(&Quest, &Chapter, &Task)> {
        let quest = self.quests.iter().find(|q| q.id == quest_id)?;
        let chapter = quest.chapters.get(chapter_idx)?;
        let task = chapter.tasks.get(task_idx)?;
        Some((quest, chapter, task))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quest {
    #[serde(alias = "name")]
    pub id: String,
    pub title: String,
    #[serde(default = "default_true")]
    pub construct: bool,
    pub chapters: Vec<Chapter>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub title: String,
    pub intro: String,
    pub outro: String,
    #[serde(default)]
    pub setup_actions: Vec<SetupAction>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub description: String,
    pub instruction: String,
    pub objective: String,
    pub success_msg: String,
    #[serde(default)]
    pub hint: String,
    pub conditions: Vec<Condition>,
    #[serde(default)]
    pub rewards: Vec<Reward>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Reward {
    SetFlag { key: String, value: bool },
    SetVar { key: String, value: i32 },
    AddVar { key: String, amount: i32 },
}

// --- LOGIC ENGINE ---

/// The result of a condition check.
#[derive(Debug, PartialEq)]
pub enum ValidationResult {
    Valid,
    SyntaxError,        // Regex failed
    LogicError(String), // State check failed (Command right, context wrong)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ConditionType {
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

    // CONTENT CHECKS
    FileContains { path: String, pattern: String },
    FileNotContains { path: String, pattern: String },
    FileEmpty { path: String },

    // ENVIRONMENT CHECKS
    WorkingDir { path: String },
    EnvVar { name: String, value: String },

    // GAME STATE CHECKS
    FlagIsTrue { key: String },
    VarEquals { key: String, value: i32 },
    VarGreaterThan { key: String, value: i32 },
    VarLessThan { key: String, value: i32 },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Condition {
    #[serde(flatten)]
    pub condition_type: ConditionType,
    pub failure_message: Option<String>,
}

impl Condition {
    fn get_sandbox_path(path: &str) -> PathBuf {
        if let Some(user_dirs) = UserDirs::new() {
            user_dirs.home_dir().join("Construct").join(path)
        } else {
            PathBuf::from(path)
        }
    }

    pub fn check(&self, user_command: &str, state: &GameState) -> ValidationResult {
        let is_valid = match &self.condition_type {
            ConditionType::CommandMatches { pattern } => {
                let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                if re.is_match(user_command) {
                    true
                } else {
                    return ValidationResult::SyntaxError;
                }
            }
            ConditionType::HistoryContains { pattern } => {
                // Simplified for brevity - assumes logic works
                let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                re.is_match("TODO_IMPLEMENT_HISTORY_READ")
            }
            // --- SANDBOXED CHECKS ---
            ConditionType::PathExists { path } => Self::get_sandbox_path(path).exists(),
            ConditionType::PathMissing { path } => !Self::get_sandbox_path(path).exists(),
            ConditionType::IsDirectory { path } => Self::get_sandbox_path(path).is_dir(),
            ConditionType::IsFile { path } => Self::get_sandbox_path(path).is_file(),
            ConditionType::IsExecutable { path } => {
                if let Ok(metadata) = fs::metadata(Self::get_sandbox_path(path)) {
                    #[cfg(unix)]
                    {
                        metadata.permissions().mode() & 0o111 != 0
                    }
                    #[cfg(not(unix))]
                    {
                        false
                    }
                } else {
                    false
                }
            }
            ConditionType::FileContains { path, pattern } => {
                if let Ok(content) = fs::read_to_string(Self::get_sandbox_path(path)) {
                    Regex::new(pattern)
                        .map(|re| re.is_match(&content))
                        .unwrap_or(false)
                } else {
                    false
                }
            }
            ConditionType::FileNotContains { path, pattern } => {
                if let Ok(content) = fs::read_to_string(Self::get_sandbox_path(path)) {
                    Regex::new(pattern)
                        .map(|re| !re.is_match(&content))
                        .unwrap_or(true)
                } else {
                    true
                }
            }
            ConditionType::FileEmpty { path } => fs::metadata(Self::get_sandbox_path(path))
                .map(|m| m.len() == 0)
                .unwrap_or(false),
            // --- ENV CHECKS ---
            ConditionType::WorkingDir { path } => {
                let current = env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                Regex::new(path)
                    .map(|re| re.is_match(&current))
                    .unwrap_or(false)
            }
            ConditionType::EnvVar { name, value } => {
                env::var(name).map(|v| v == *value).unwrap_or(false)
            }

            // --- STATE CHECKS ---
            ConditionType::FlagIsTrue { key } => state.get_flag(key),
            ConditionType::VarEquals { key, value } => state.get_var(key) == *value,
            ConditionType::VarGreaterThan { key, value } => state.get_var(key) > *value,
            ConditionType::VarLessThan { key, value } => state.get_var(key) < *value,
        };

        if is_valid {
            ValidationResult::Valid
        } else {
            // It failed. Was it syntax (already handled) or logic?
            // If we are here, it's a Logic Error.
            let msg = self
                .failure_message
                .clone()
                .unwrap_or_else(|| "Condition not met.".to_string());
            ValidationResult::LogicError(msg)
        }
    }
}
