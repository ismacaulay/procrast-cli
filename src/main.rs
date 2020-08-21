mod cmd;
mod db;
mod models;

use db::Database;

// use std::env;

fn main() {
    let db = db::sqlite::new();
    println!("{:?}", db.get_lists());

    let list_name = String::from("foobar");
    db.create_list(&list_name);
    println!("{:?}", db.get_lists());

    match db.get_list(&list_name) {
        Some(list) => println!("List Found! {:?}", list),
        None => println!("List not found!"),
    }
    // let args: Vec<String> = env::args().collect();
    // cmd::process(&args[1..]);
}
