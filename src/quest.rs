use directories::UserDirs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

// --- TIER 1: THE ROOT ---
#[derive(Debug, Serialize, Deserialize)]
pub struct Curriculum {
    pub quests: Vec<Quest>,
}

// --- TIER 2: THE SEASON ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub chapters: Vec<Chapter>,
}

// --- TIER 3: THE EPISODE ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub title: String,
    pub intro: String,
    pub outro: String,
    pub tasks: Vec<Task>,
}

// --- TIER 4: THE SCENE ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub description: String,
    pub instruction: String,
    pub objective: String,
    pub success_msg: String,
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

impl Curriculum {
    /// Loads the full hierarchy from the YAML file.
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_yml::from_str(&content).unwrap_or_else(|e| {
                eprintln!("Failed to parse quests.yaml: {}", e);
                Curriculum { quests: vec![] }
            })
        } else {
            Curriculum { quests: vec![] }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir; // You'll need to add 'tempfile' to dev-dependencies

    #[test]
    fn test_regex_condition() {
        let cond = Condition::CommandMatches {
            pattern: "^git commit".to_string(),
        };

        assert!(cond.is_met("git commit -m 'fix'"));
        assert!(!cond.is_met("git add ."));
    }

    #[test]
    fn test_file_exists_condition() {
        // Create a temporary directory that cleans up after itself
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");

        // 1. Assert file is missing initially
        let cond = Condition::PathExists {
            path: file_path.to_str().unwrap().to_string(),
        };
        assert!(!cond.is_met("")); // User command doesn't matter for path checks

        // 2. Create file
        File::create(&file_path).unwrap();

        // 3. Assert condition passes
        assert!(cond.is_met(""));
    }
}
