use crate::config;
use std::{env, fs, io::Read, process};

pub fn get_file_input() -> String {
    // TODO: how do we handle multiple platforms
    // TODO: what happens if EDITOR and nano dont exist
    // TODO: Handle errors better
    let editor = match env::var("EDITOR") {
        Ok(e) => e,
        Err(_) => String::from("nano"),
    };

    let mut file_path = config::get_data_dir().expect("Failed to get data dir");
    file_path.push("PROCRAST_MESSAGE");

    fs::File::create(&file_path).expect("Could not create the file");

    process::Command::new(editor)
        .arg(&file_path)
        .status()
        .expect("Something went wrong");

    let mut message = String::new();
    let mut file = fs::File::open(&file_path).expect("Could not open file");
    file.read_to_string(&mut message)
        .expect("Failed to read file");

    // clear file contents after since these are suppose to be private thoughts
    fs::File::create(&file_path).expect("Could not create file");

    return message;
}
