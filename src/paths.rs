use directories::ProjectDirs;
use std::path::PathBuf;

pub struct AppContext {
    pub library_path: PathBuf,
    pub save_path: PathBuf,
}

pub fn build_app_context() -> AppContext {
    let data_dir = if std::env::var("SUPERSHELL_TEST_MODE").is_ok() {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("supershell-test"))
            .join("supershell")
    } else {
        let proj_dirs = ProjectDirs::from("com", "jalexlong", "supershell")
            .expect("Could not determine home directory");

        proj_dirs.data_dir().to_path_buf()
    };

    AppContext {
        library_path: data_dir.join("library"),
        save_path: data_dir.join("save.json"),
    }
}
