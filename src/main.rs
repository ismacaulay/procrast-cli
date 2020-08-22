mod cmd;
mod db;
mod models;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    cmd::process(&args[1..]);
}
