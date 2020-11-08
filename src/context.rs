use crate::{config, models, sqlite};
use reqwest;
use rusqlite;
use std::collections::HashMap;

pub struct Context {
    pub db: rusqlite::Connection,
    pub client: reqwest::blocking::Client,
    pub config: models::Config,
    pub data: HashMap<&'static str, String>,
    pub params: Vec<String>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            db: sqlite::new(),
            client: reqwest::blocking::Client::new(),
            config: config::load().unwrap(),
            data: HashMap::new(),
            params: Vec::new(),
        }
    }
}
