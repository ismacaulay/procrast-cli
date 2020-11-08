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
    pub next_note_id: i32,
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

#[derive(Debug)]
pub struct Note {
    pub uuid: uuid::Uuid,
    pub id: i32,
    pub title: String,
    pub body: String,
    pub created: i64,
    pub modified: i64,
    pub list_uuid: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    pub uuid: uuid::Uuid,
    pub command: String,
    pub state: String,
    pub timestamp: i64,
    pub synced: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiHistory {
    pub uuid: uuid::Uuid,
    pub command: String,
    pub state: String,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
pub struct ApiHistoryCreatedState {
    pub uuid: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CmdListState {
    pub uuid: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub created: i64,
    pub modified: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CmdDeleteState {
    pub uuid: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CmdItemState {
    pub uuid: uuid::Uuid,
    pub title: String,
    pub description: String,
    pub state: i8,
    pub created: i64,
    pub modified: i64,
    pub list_uuid: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CmdNoteState {
    pub uuid: uuid::Uuid,
    pub title: String,
    pub body: String,
    pub created: i64,
    pub modified: i64,
    pub list_uuid: uuid::Uuid,
}

pub const CMD_LIST_CREATE: &'static str = "LIST CREATE";
pub const CMD_LIST_UPDATE: &'static str = "LIST UPDATE";
pub const CMD_LIST_DELETE: &'static str = "LIST DELETE";
pub const CMD_ITEM_CREATE: &'static str = "ITEM CREATE";
pub const CMD_ITEM_UPDATE: &'static str = "ITEM UPDATE";
pub const CMD_ITEM_DELETE: &'static str = "ITEM DELETE";
pub const CMD_NOTE_CREATE: &'static str = "NOTE CREATE";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub base_url: String,
    pub token: String,
}
