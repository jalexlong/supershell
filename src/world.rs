use crate::actions::SetupAction;
use directories::UserDirs;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

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
                    let Some(target) = self.safe_path(path) else {
                        eprintln!(">> [WORLD] Refusing unsafe directory path: {path}");
                        continue;
                    };

                    fs::create_dir_all(target).unwrap_or_else(|e| {
                        eprintln!("Failed to create dir: {}", e);
                    });
                }
                SetupAction::CreateFile { path, content } => {
                    let Some(target) = self.safe_path(path) else {
                        eprintln!(">> [WORLD] Refusing unsafe file path: {path}");
                        continue;
                    };

                    // Ensure parent directory exists first.
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent).ok();
                    }

                    match fs::File::create(&target) {
                        Ok(mut file) => {
                            if let Err(err) = file.write_all(content.as_bytes()) {
                                eprintln!("Failed to write content to {:?}: {}", target, err);
                            }
                        }
                        Err(err) => {
                            eprintln!("Failed to create file {:?}: {}", target, err);
                        }
                    }
                }
                SetupAction::RemovePath { path } => {
                    let Some(target) = self.safe_path(path) else {
                        eprintln!(">> [WORLD] Refusing unsafe removal path: {path}");
                        continue;
                    };

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

    /// Safely joins a YAML-provided relative path to the Construct root.
    ///
    /// Rejects:
    /// - empty paths
    /// - absolute paths
    /// - Windows path prefixes
    /// - parent directory traversal using `..`
    ///
    /// This keeps setup actions inside ~/Construct.
    fn safe_path(&self, relative_path: &str) -> Option<PathBuf> {
        let path = Path::new(relative_path);

        if relative_path.trim().is_empty() || path.is_absolute() {
            return None;
        }

        if path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        }) {
            return None;
        }

        Some(self.root_path.join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_world() -> WorldEngine {
        WorldEngine {
            root_path: PathBuf::from("/tmp/supershell-test-construct"),
        }
    }

    #[test]
    fn safe_path_allows_normal_relative_paths() {
        let world = test_world();

        assert_eq!(
            world.safe_path("Memory_Bank/welcome.txt"),
            Some(PathBuf::from(
                "/tmp/supershell-test-construct/Memory_Bank/welcome.txt"
            ))
        );
    }

    #[test]
    fn safe_path_rejects_parent_directory_traversal() {
        let world = test_world();

        assert_eq!(world.safe_path("../outside.txt"), None);
        assert_eq!(world.safe_path("Memory_Bank/../../outside.txt"), None);
    }

    #[test]
    fn safe_path_rejects_absolute_paths() {
        let world = test_world();

        assert_eq!(world.safe_path("/tmp/outside.txt"), None);
    }

    #[test]
    fn safe_path_rejects_empty_paths() {
        let world = test_world();

        assert_eq!(world.safe_path(""), None);
        assert_eq!(world.safe_path("   "), None);
    }
}
