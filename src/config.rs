use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

pub fn get_data_dir() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "ismacaul", "procrast") {
        if !proj_dirs.data_dir().exists() {
            fs::create_dir_all(proj_dirs.data_dir()).expect("Failed to create data dir");
        }
        return Some(proj_dirs.data_dir().to_path_buf());
    }
    return None;
}

pub fn get_config_dir() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "ismacaul", "procrast") {
        if !proj_dirs.config_dir().exists() {
            fs::create_dir_all(proj_dirs.config_dir()).expect("Failed to create data dir");
        }
        return Some(proj_dirs.config_dir().to_path_buf());
    }
    return None;
}

// TODO: Probably should version the config
#[derive(Deserialize, Serialize)]
struct Config {
    current_list: String,
}

fn create_config_file() -> Option<fs::File> {
    let mut config_path_buf = get_config_dir().expect("Failed to get config dir path");
    config_path_buf.push("config.json");
    let config_path = Path::new(&config_path_buf);

    if let Ok(file) = fs::File::create(config_path) {
        return Some(file);
    }

    return None;
}

fn get_config_file() -> Option<fs::File> {
    let mut config_path_buf = get_config_dir().expect("Failed to get config dir path");
    config_path_buf.push("config.json");
    let config_path = Path::new(&config_path_buf);

    if config_path.exists() {
        Some(fs::File::open(config_path).expect("Failed to open config file"));
    }
    return None;
}

// TODO: this kind of sucks but works... maybe make it better
pub fn set_current_list(id: &String) {
    if let Some(config_file) = get_config_file() {
        let reader = BufReader::new(config_file);
        let mut config: Config =
            serde_json::from_reader(reader).expect("Could not read config from reader");
        config.current_list = id.to_string();

        let writer = BufWriter::new(get_config_file().unwrap());
        serde_json::to_writer(writer, &config).expect("Could not write config file");
    } else if let Some(config_file) = create_config_file().as_mut() {
        let config = Config {
            current_list: id.to_string(),
        };
        let config_str = serde_json::to_string_pretty(&config).expect("Could not stringify config");
        config_file
            .write_all(config_str.as_bytes())
            .expect("Failed to write config file");
    } else {
        println!("Failed to write config file");
        std::process::exit(1);
    }
}
