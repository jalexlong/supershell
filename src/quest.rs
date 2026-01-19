use crate::actions::SetupAction;
use directories::UserDirs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

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
                // Check for .yaml or .yml
                if let Some(ext) = path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        let course = Course::load(&path);

                        // Fallback to filename if title is "Untitled Course" (optional)
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
        // Sort by title alphabetically
        courses.sort_by(|a, b| a.1.cmp(&b.1));
        courses
    }
}

// --- TIER 1: THE COURSE ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Course {
    // METADATA
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default = "default_version")]
    pub version: String, // e.g., "1.0.0"

    // CONTENT
    pub quests: Vec<Quest>,
}

// These helper functions allow old YAML files to load without crashing
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

        // ATTEMPT 1: Standard "Wrapped" Format (v0.5.0 Standard)
        // This looks for: { title: "...", quests: [...] }
        // The #serde(default)] attributes on the struct handle missing fields here.
        match serde_yml::from_str::<Course>(&content) {
            Ok(course) => return course,
            Err(e1) => {
                // ATTEMPT 2: Legacy List Format (v0.4.0)
                // This looks for: [ {id: ... }, { id: ...} ]
                match serde_yml::from_str::<Vec<Quest>>(&content) {
                    Ok(quests) => {
                        // MANUAL UPGRADE: Wrap the old list in the new struct
                        return Course {
                            title: default_title(),
                            author: default_author(),
                            version: default_version(),
                            quests,
                        };
                    }
                    Err(e2) => {
                        // ATTEMPT 3: Legacy Single Object (v0.1.0)
                        // This looks for: { id: ..., chapters: ... }
                        match serde_yml::from_str::<Quest>(&content) {
                            Ok(quest) => {
                                return Course {
                                    title: default_title(),
                                    author: default_author(),
                                    version: default_version(),
                                    quests: vec![quest],
                                };
                            }
                            Err(e3) => {
                                // CRASH HERE so we can see the error
                                panic!(
                                    "\n\n[YAML PARSE ERROR]\nFile: {:?}\nAttempt 1 (Course): {}\nAttempt 2 (List): {}\nAttempt 3 (Single): {}\n\n",
                                    path, e1, e2, e3
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// Finds the specific Quest/Chapter/Task based on indices or IDs
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

    /// Look ahead logic: Returns the info for the NEXT step (if it exists)
    /// Handles crossing Chapter boundaries
    pub fn find_next_step(
        &self,
        quest_id: &str,
        current_chapter_idx: usize,
        current_task_idx: usize,
    ) -> Option<NextStepInfo> {
        let quest = self.quests.iter().find(|q| q.id == quest_id)?;
        let current_chapter = quest.chapters.get(current_chapter_idx)?;

        // Case A: Next Task in SAME Chapter
        if current_task_idx + 1 < current_chapter.tasks.len() {
            let task = &current_chapter.tasks[current_task_idx + 1];
            return Some(NextStepInfo {
                instruction: task.instruction.clone(),
                objective: task.objective.clone(),
            });
        }

        // Case B: First Task in NEXT Chapter
        if current_chapter_idx + 1 < quest.chapters.len() {
            let next_chapter = &quest.chapters[current_chapter_idx + 1];
            if let Some(first_task) = next_chapter.tasks.first() {
                return Some(NextStepInfo {
                    instruction: first_task.instruction.clone(),
                    objective: first_task.objective.clone(),
                });
            }
        }
        // Case C: Quest Complete (No next step)
        None
    }
}

pub struct NextStepInfo {
    pub instruction: String,
    pub objective: String,
}

// --- TIER 2: THE SEASON ---
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

// --- TIER 3: THE EPISODE ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub title: String,
    pub intro: String,
    pub outro: String,
    #[serde(default)]
    pub setup_actions: Vec<SetupAction>,
    pub tasks: Vec<Task>,
}

// --- TIER 4: THE SCENE ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub description: String,
    pub instruction: String,
    pub objective: String,
    pub success_msg: String,
    #[serde(default)]
    pub hint: String,
    pub conditions: Vec<Condition>,
}

// --- LOGIC: CONDITIONS ---
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
    WorkingDir { path: String },

    // CONTENT CHECKS
    FileContains { path: String, pattern: String },
    FileNotContains { path: String, pattern: String },
    FileEmpty { path: String },

    // ENVIRONMENT CHECKS
    EnvVar { name: String, value: String },
}

impl Condition {
    /// Helper to get the Sandbox Root (~/Construct)
    fn get_sandbox_path(path: &str) -> PathBuf {
        if let Some(user_dirs) = UserDirs::new() {
            user_dirs.home_dir().join("Construct").join(path)
        } else {
            PathBuf::from(path) // Fallback (shouldn't happen)
        }
    }

    pub fn is_met(&self, user_command: &str) -> bool {
        match self {
            Condition::CommandMatches { pattern } => {
                let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                re.is_match(user_command)
            }
            Condition::HistoryContains { pattern } => {
                if let Some(user_dirs) = UserDirs::new() {
                    // Check standard bash/zsh history locations
                    let history_files = vec![".bash_history", ".zsh_history"];
                    for fname in history_files {
                        let history_path = user_dirs.home_dir().join(fname);
                        if let Ok(content) = fs::read_to_string(history_path) {
                            let re =
                                Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                            if re.is_match(&content) {
                                return true;
                            }
                        }
                    }
                    false
                } else {
                    false
                }
            }
            // --- SANDBOXED CHECKS ---
            Condition::PathExists { path } => Self::get_sandbox_path(path).exists(),
            Condition::PathMissing { path } => !Self::get_sandbox_path(path).exists(),
            Condition::IsDirectory { path } => Self::get_sandbox_path(path).is_dir(),
            Condition::IsFile { path } => Self::get_sandbox_path(path).is_file(),
            Condition::IsExecutable { path } => {
                if let Ok(metadata) = fs::metadata(Self::get_sandbox_path(path)) {
                    metadata.permissions().mode() & 0o111 != 0
                } else {
                    false
                }
            }
            Condition::FileContains { path, pattern } => {
                if let Ok(content) = fs::read_to_string(Self::get_sandbox_path(path)) {
                    let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                    re.is_match(&content)
                } else {
                    false
                }
            }
            Condition::FileNotContains { path, pattern } => {
                if let Ok(content) = fs::read_to_string(Self::get_sandbox_path(path)) {
                    let re = Regex::new(pattern).unwrap_or_else(|_| Regex::new("").unwrap());
                    !re.is_match(&content)
                } else {
                    true // If file doesn't exist, it certainly doesn't contain the pattern
                }
            }
            Condition::FileEmpty { path } => {
                if let Ok(metadata) = fs::metadata(Self::get_sandbox_path(path)) {
                    metadata.len() == 0
                } else {
                    false
                }
            }
            // --- NON-SANDBOXED CHECKS ---
            Condition::WorkingDir { path } => {
                let current_dir = env::current_dir().unwrap_or_default();
                let path_str = current_dir.to_string_lossy();
                // We use the 'path' field as a Regex Pattern
                let re = Regex::new(path).unwrap_or_else(|_| Regex::new("").unwrap());
                re.is_match(&path_str)
            }
            Condition::EnvVar { name, value } => match env::var(name) {
                Ok(val) => val == *value,
                Err(_) => false,
            },
        }
    }
}
