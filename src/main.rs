mod cmd;
mod config;
mod db;
mod input;
mod models;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    cmd::process(&args[1..]);
}
