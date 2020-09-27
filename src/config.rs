use crate::{models, utils::Result};
#[cfg(production)]
use directories::ProjectDirs;
use std::{
    fs,
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

#[cfg(production)]
pub fn get_data_dir() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "ismacaul", "procrast") {
        if !proj_dirs.data_dir().exists() {
            fs::create_dir_all(proj_dirs.data_dir()).expect("Failed to create data dir");
        }
        return Some(proj_dirs.data_dir().to_path_buf());
    }
    return None;
}

#[cfg(not(production))]
pub fn get_data_dir() -> Option<PathBuf> {
    let mut local_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    local_dir.push(".local");

    if !local_dir.exists() {
        fs::create_dir_all(&local_dir).expect("Failed to create data dir");
    }

    return Some(local_dir);
}

#[cfg(production)]
pub fn get_config_dir() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "ismacaul", "procrast") {
        if !proj_dirs.config_dir().exists() {
            fs::create_dir_all(proj_dirs.config_dir()).expect("Failed to create data dir");
        }
        return Some(proj_dirs.config_dir().to_path_buf());
    }
    return None;
}

#[cfg(not(production))]
pub fn get_config_dir() -> Option<PathBuf> {
    let mut local_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    local_dir.push(".local");

    if !local_dir.exists() {
        fs::create_dir_all(&local_dir).expect("Failed to create data dir");
    }

    return Some(local_dir);
}

fn create_config_file() -> Result<()> {
    let mut config_path_buf = get_config_dir().expect("Failed to get config dir path");
    config_path_buf.push("config.json");
    let config_path = Path::new(&config_path_buf);

    if !config_path.exists() {
        match fs::File::create(config_path) {
            Ok(mut f) => {
                let config = models::Config {
                    base_url: String::new(),
                    token: String::new(),
                };
                let config_str =
                    serde_json::to_string_pretty(&config).expect("Could not stringify config");

                if let Err(e) = f.write_all(config_str.as_bytes()) {
                    return Err(format!("Failed to write config file: {}", e));
                }
            }
            Err(e) => return Err(format!("Failed to create config file: {}", e)),
        };
    }

    Ok(())
}

pub fn load() -> Result<models::Config> {
    let mut config_path_buf = get_config_dir().expect("Failed to get config dir path");
    config_path_buf.push("config.json");
    let config_path = Path::new(&config_path_buf);
    if !config_path.exists() {
        create_config_file()?;
    }

    let config_file = fs::File::open(config_path).expect("failed to open config file");
    let reader = BufReader::new(config_file);

    match serde_json::from_reader(reader) {
        Ok(config) => Ok(config),
        Err(e) => Err(format!("failed to deserialize config: {}", e)),
    }
}

pub fn save(config: &models::Config) -> Result<()> {
    let mut config_path_buf = get_config_dir().expect("Failed to get config dir path");
    config_path_buf.push("config.json");
    let config_path = Path::new(&config_path_buf);

    let config_str = serde_json::to_string_pretty(config).expect("Could not stringify config");

    let mut config_file = fs::File::create(config_path).expect("failed to open config file");
    match config_file.write_all(config_str.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to write config file: {}", e)),
    }
}
