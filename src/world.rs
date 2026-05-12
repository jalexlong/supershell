use crate::actions::SetupAction;
use crate::construct::{default_construct_root, resolve_construct_path};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct WorldEngine {
    root_path: PathBuf,
}

impl WorldEngine {
    pub fn new() -> Self {
        let root = default_construct_root().expect("Critical: Could not find User Home.");

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
                    self.reset_world();
                }
            }
        }
    }

    fn reset_world(&self) {
        if !self.root_path.ends_with("Construct") {
            eprintln!(
                ">> [WORLD] Refusing to reset suspicious world root: {:?}",
                self.root_path
            );
            return;
        }

        if !self.root_path.exists() {
            return;
        }

        let entries = match fs::read_dir(&self.root_path) {
            Ok(entries) => entries,
            Err(err) => {
                eprintln!(
                    ">> [WORLD] Failed to read Construct directory {:?}: {}",
                    self.root_path, err
                );
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    eprintln!(">> [WORLD] Failed to inspect Construct entry: {err}");
                    continue;
                }
            };

            let path = entry.path();

            let result = if path.is_dir() {
                fs::remove_dir_all(&path)
            } else {
                fs::remove_file(&path)
            };

            if let Err(err) = result {
                eprintln!(">> [WORLD] Failed to remove {:?}: {}", path, err);
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
        resolve_construct_path(&self.root_path, relative_path)
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
