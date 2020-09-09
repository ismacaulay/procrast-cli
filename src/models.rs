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
