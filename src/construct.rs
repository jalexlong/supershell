use directories::UserDirs;
use std::path::{Component, Path, PathBuf};

pub fn default_construct_root() -> Option<PathBuf> {
    UserDirs::new().map(|user_dirs| user_dirs.home_dir().join("Construct"))
}

pub fn resolve_construct_path(root: &Path, relative_path: &str) -> Option<PathBuf> {
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

    Some(root.join(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_root() -> PathBuf {
        PathBuf::from("/tmp/supershell-test-construct")
    }

    #[test]
    fn resolve_construct_path_allows_normal_relative_paths() {
        assert_eq!(
            resolve_construct_path(&test_root(), "Memory_Bank/welcome.txt"),
            Some(PathBuf::from(
                "/tmp/supershell-test-construct/Memory_Bank/welcome.txt"
            ))
        );
    }

    #[test]
    fn resolve_construct_path_rejects_parent_directory_traversal() {
        assert_eq!(resolve_construct_path(&test_root(), "../outside.txt"), None);
        assert_eq!(
            resolve_construct_path(&test_root(), "Memory_Bank/../../outside.txt"),
            None
        );
    }

    #[test]
    fn resolve_construct_path_rejects_absolute_paths() {
        assert_eq!(
            resolve_construct_path(&test_root(), "/tmp/outside.txt"),
            None
        );
    }

    #[test]
    fn resolve_construct_path_rejects_empty_paths() {
        assert_eq!(resolve_construct_path(&test_root(), ""), None);
        assert_eq!(resolve_construct_path(&test_root(), "   "), None);
    }
}
