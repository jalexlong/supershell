use crate::actions::SetupAction;
use directories::UserDirs;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct WorldEngine {
    root_path: PathBuf,
}

impl WorldEngine {
    pub fn new() -> Self {
        // 1. Locate the User's Home Directory safely
        let user_dirs = UserDirs::new().expect("Critical: Could not find User Home.");
        let home_dir = user_dirs.home_dir();

        // 2. Target "~/Construct"
        // This works cross-platform (C:\Users\Name\Construct or /home/name/Construct)
        let root = home_dir.join("Construct");

        WorldEngine { root_path: root }
    }

    /// Run this once on startup to ensure the "Construct" folder exists
    pub fn initialize(&self) {
        if !self.root_path.exists() {
            println!(">> [WORLD] Initializing Construct at {:?}", self.root_path);
            fs::create_dir_all(&self.root_path).expect("Failed to create world root.");
        }
    }

    /// The Main Loop: Reads YAML instructions and executes them
    pub fn build_scenario(&self, actions: &[SetupAction]) {
        for action in actions {
            match action {
                SetupAction::CreateDir { path } => {
                    let target = self.safe_path(path);
                    fs::create_dir_all(target).unwrap_or_else(|e| {
                        eprintln!("Failed to create dir: {}", e);
                    });
                }
                SetupAction::CreateFile { path, content } => {
                    let target = self.safe_path(path);

                    // Ensure parent directory exists first!
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent).ok();
                    }

                    let mut file = fs::File::create(target).expect("Failed to create file");
                    file.write_all(content.as_bytes())
                        .expect("Failed to write content");
                }
                SetupAction::RemovePath { path } => {
                    let target = self.safe_path(path);
                    if target.exists() {
                        if target.is_dir() {
                            fs::remove_dir_all(target).ok();
                        } else {
                            fs::remove_file(target).ok();
                        }
                    }
                }
                SetupAction::ResetWorld => {
                    if self.root_path.ends_with("Construct") && self.root_path.exists() {
                        for entry in fs::read_dir(&self.root_path).unwrap() {
                            let entry = entry.unwrap();
                            let path = entry.path();
                            if path.is_dir() {
                                fs::remove_dir_all(path).ok();
                            } else {
                                fs::remove_file(path).ok();
                            }
                        }
                    }
                }
            }
        }
    }

    /// SAFETY: Joins the user input to ~/Construct
    /// Prevents users from writing "setup_action: ../../System32"
    fn safe_path(&self, relative_path: &str) -> PathBuf {
        // A real production app needs ".." sanitization here.
        // For now, we trust the YAML writer (you).
        self.root_path.join(relative_path)
    }
}
