use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct List {
    pub uuid: uuid::Uuid,
    pub id: i32,
    pub title: String,
    pub description: String,
    pub created: i64,
    pub modified: i64,
    pub next_item_id: i32,
}

#[derive(Debug)]
pub struct Item {
    pub uuid: uuid::Uuid,
    pub id: i32,
    pub title: String,
    pub description: String,
    pub state: i8,
    pub created: i64,
    pub modified: i64,
    pub list_uuid: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    pub uuid: uuid::Uuid,
    pub command: String,
    pub state: String,
    pub created: i64,
    pub synced: bool,
}

#[derive(Debug, Deserialize)]
pub struct ApiHistory {
    pub uuid: uuid::Uuid,
    pub command: String,
    pub state: String,
    pub created: i64,
}

#[derive(Debug, Deserialize)]
pub struct ApiList {
    pub uuid: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub created: i64,
    pub modified: i64,
}

#[derive(Debug, Deserialize)]
pub struct CmdListDeleteState {
    pub uuid: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ApiItem {
    pub uuid: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub created: i64,
    pub modified: i64,
    pub list_uuid: uuid::Uuid,
}
