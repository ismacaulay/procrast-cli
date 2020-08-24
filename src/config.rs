use directories::ProjectDirs;
use std::{fs, path::PathBuf};

pub fn get_data_dir() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "ismacaul", "procrast") {
        if !proj_dirs.data_dir().exists() {
            fs::create_dir_all(proj_dirs.data_dir()).expect("Failed to create data dir");
        }
        return Some(proj_dirs.data_dir().to_path_buf());
    }
    return None;
}
