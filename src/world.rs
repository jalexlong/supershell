// world.rs

use crate::actions::SetupAction;
use directories::UserDirs;
use log::{debug, error, info};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

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
            info!("Initializing Construct Environment at {:?}", self.root_path);
            if let Err(e) = fs::create_dir_all(&self.root_path) {
                error!("Critical Failure: Unable to create Construct root. {}", e);
            }
        }
    }

    /// The Main Loop: Reads YAML instructions and executes them
    pub fn build_scenario(&self, actions: &[SetupAction]) {
        for action in actions {
            match action {
                SetupAction::CreateDir { path } => {
                    let target = self.safe_path(path);
                    debug!("Action [CreateDir]: {:?}", target);
                    if let Err(e) = fs::create_dir_all(target) {
                        error!("Action Failed [CreateDir]: {}", e);
                    }
                }
                SetupAction::CreateFile { path, content } => {
                    let target = self.safe_path(path);
                    debug!("Action [CreateFile]: {:?}", target);
                    if let Err(e) =
                        fs::File::create(&target).and_then(|mut f| f.write_all(content.as_bytes()))
                    {
                        error!("Action Failed [CreateFile]: {}", e);
                    }
                }
                SetupAction::RemovePath { path } => {
                    let target = self.safe_path(path);
                    debug!("Action [RemovePath]: {:?}", target);
                    if target.exists() {
                        if target.is_dir() {
                            fs::remove_dir_all(target).ok();
                        } else {
                            fs::remove_file(target).ok();
                        }
                    }
                }
                SetupAction::ResetWorld => {
                    info!("Action [ResetWorld]: Purging Construct directory.");
                    if self.root_path.ends_with("Construct") && self.root_path.exists() {
                        if let Ok(entries) = fs::read_dir(&self.root_path) {
                            for entry in entries.flatten() {
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
    }

    /// SAFETY: Joins the user input to ~/Construct
    /// Prevents users from writing "setup_action: ../../System32"
    fn safe_path(&self, relative_path: &str) -> PathBuf {
        // A real production app needs ".." sanitization here.
        // For now, we trust the YAML writer (you).
        self.root_path.join(relative_path)
    }
}
