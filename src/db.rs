use crate::models;

pub trait Database {
    fn get_lists(&self) -> Vec<models::List>;
    fn get_current_list(&self) -> Option<String>;
    fn set_current_list(&self, list: &String);

    fn create_list(&self, title: &String, description: &String);
    fn update_list(&self, list: &models::List);
    fn delete_list(&self, list: &models::List);
    fn find_list_by_title(&self, title: &String) -> Option<models::List>;
    fn find_list_by_id(&self, id: &String) -> Option<models::List>;

    fn get_items(&self, list_id: &String) -> Vec<models::Item>;
    fn get_item(&self, list_id: &String, item_id: &String) -> Option<models::Item>;
    fn create_item(&self, list_id: &String, title: &String, description: &String);
    fn update_item(&self, list_id: &String, item: &models::Item);
    fn delete_item(&self, list_id: &String, item: &models::Item);
}

pub mod sqlite {
    use crate::{config, models};
    use rusqlite::{params, Connection, NO_PARAMS};
    use std::path::{Path, PathBuf};

    fn get_database_path(name: &str) -> Option<PathBuf> {
        if let Some(data_dir) = config::get_data_dir() {
            return Some(data_dir.join(name));
        }
        return None;
    }

    pub struct SQLiteDatabase {
        conn: rusqlite::Connection,
    }

    pub fn new() -> SQLiteDatabase {
        let db_path_buf = get_database_path("db.sqlite").expect("Failed to get database path");
        let db_path = Path::new(&db_path_buf);
        let new_database = !db_path.exists();

        let conn = Connection::open(&db_path).expect("Failed to open db");
        if new_database {
            conn.execute(
                "CREATE TABLE config (
                id INTEGER PRIMARY KEY CHECK (id = 0),
                current_list TEXT)",
                NO_PARAMS,
            )
            .expect("Failed to create list table");
            conn.execute(
                "INSERT INTO config (id, current_list) VALUES (0, ?1)",
                params!["1"],
            )
            .expect("Failed to set default list");

            conn.execute(
                "CREATE TABLE lists (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT)",
                NO_PARAMS,
            )
            .expect("Failed to create list table");

            conn.execute(
                "CREATE TABLE items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                list_id INTEGER REFERENCES lists(id))",
                NO_PARAMS,
            )
            .expect("Failed to create item table");

            conn.execute("INSERT INTO lists (title) VALUES (?1)", params!["todo"])
                .expect("Failed to create default list");
        }

        SQLiteDatabase { conn: conn }
    }

    impl crate::db::Database for SQLiteDatabase {
        fn get_lists(&self) -> Vec<models::List> {
            let mut stmt = self
                .conn
                .prepare("SELECT id, title, description FROM lists")
                .expect("Failed to prepare query");

            let list_iter = stmt
                .query_map(NO_PARAMS, |row| {
                    let desc = match row.get::<_, String>(2) {
                        Ok(desc) => desc,
                        _ => String::from(""),
                    };
                    Ok(models::List {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: desc,
                    })
                })
                .expect("Failed to perform query_map");

            let mut lists = Vec::new();
            for l in list_iter {
                lists.push(l.unwrap());
            }
            return lists;
        }

        fn get_current_list(&self) -> Option<String> {
            let result = self.conn.query_row(
                "SELECT current_list FROM config WHERE id = 0",
                NO_PARAMS,
                |row| Ok(row.get(0)?),
            );
            match result {
                Ok(data) => Some(data),
                Err(_) => None,
            }
        }

        fn set_current_list(&self, list: &String) {
            self.conn
                .execute(
                    "UPDATE config SET current_list = ?1 WHERE id = 0",
                    params![list],
                )
                .expect("Failed to update list");
        }

        fn create_list(&self, title: &String, description: &String) {
            self.conn
                .execute(
                    "INSERT INTO lists (title, description) VALUES (?1, ?2)",
                    params![title, description],
                )
                .expect("Failed to create list");
        }

        fn update_list(&self, list: &models::List) {
            self.conn
                .execute(
                    "UPDATE lists SET title = ?2, description = ?3 WHERE id = ?1",
                    params![list.id, list.title, list.description],
                )
                .expect("Failed to update list");
        }

        fn delete_list(&self, list: &models::List) {
            self.conn
                .execute("DELETE FROM lists WHERE id = ?1", params![list.id])
                .expect("Failed to delete list");
        }

        fn find_list_by_title(&self, title: &String) -> Option<models::List> {
            let result = self.conn.query_row(
                "SELECT id, title, description FROM lists WHERE title = (?1)",
                params![title],
                |row| {
                    let desc = match row.get::<_, String>(2) {
                        Ok(desc) => desc,
                        _ => String::from(""),
                    };
                    Ok(models::List {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: desc,
                    })
                },
            );

            match result {
                Ok(data) => Some(data),
                Err(_) => None,
            }
        }

        fn find_list_by_id(&self, id: &String) -> Option<models::List> {
            let result = self.conn.query_row(
                "SELECT id, title, description FROM lists WHERE id = (?1)",
                params![id],
                |row| {
                    let desc = match row.get::<_, String>(2) {
                        Ok(desc) => desc,
                        _ => String::from(""),
                    };
                    Ok(models::List {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: desc,
                    })
                },
            );

            match result {
                Ok(data) => Some(data),
                Err(_) => None,
            }
        }

        fn get_items(&self, list_id: &String) -> Vec<models::Item> {
            let mut stmt = self
                .conn
                .prepare("SELECT id, title, description FROM items WHERE list_id = ?1")
                .expect("Failed to prepare query");

            let iter = stmt
                .query_map(params![list_id], |row| {
                    let desc = match row.get::<_, String>(2) {
                        Ok(desc) => desc,
                        _ => String::from(""),
                    };
                    Ok(models::Item {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: desc,
                    })
                })
                .expect("Failed to perform query_map");

            let mut items = Vec::new();
            for v in iter {
                items.push(v.unwrap());
            }
            return items;
        }

        fn get_item(&self, list_id: &String, item_id: &String) -> Option<models::Item> {
            let result = self.conn.query_row(
                "SELECT id, title, description FROM items WHERE list_id = (?1) AND id = (?2)",
                params![list_id, item_id],
                |row| {
                    let desc = match row.get::<_, String>(2) {
                        Ok(desc) => desc,
                        _ => String::from(""),
                    };
                    Ok(models::Item {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: desc,
                    })
                },
            );

            match result {
                Ok(data) => Some(data),
                Err(_) => None,
            }
        }

        fn create_item(&self, list_id: &String, title: &String, description: &String) {
            self.conn
                .execute(
                    "INSERT INTO items (title, description, list_id) VALUES (?1, ?2, ?3)",
                    params![title, description, list_id],
                )
                .expect("Failed to create item");
        }

        fn update_item(&self, list_id: &String, item: &models::Item) {
            self.conn
                .execute(
                    "UPDATE items SET title = ?3, description = ?4 WHERE list_id = ?1 AND id = ?2",
                    params![list_id, item.id, item.title, item.description],
                )
                .expect("Failed to update list");
        }

        fn delete_item(&self, list_id: &String, item: &models::Item) {
            self.conn
                .execute(
                    "DELETE FROM items WHERE list_id = ?1 AND id = ?2",
                    params![list_id, item.id],
                )
                .expect("Failed to delete item");
        }
    }
}

pub fn get_current_list() -> Option<String> {
    let db = sqlite::new();
    return db.get_current_list();
}

pub fn set_current_list(list: &String) {
    let db = sqlite::new();
    return db.set_current_list(list);
}

pub fn get_lists() -> Vec<models::List> {
    let db = sqlite::new();
    return db.get_lists();
}

pub fn create_list(title: &String, description: &String) {
    let db = sqlite::new();
    db.create_list(title, description);
}

pub fn update_list(list: &models::List) {
    let db = sqlite::new();
    db.update_list(list);
}

pub fn delete_list(list: &models::List) {
    let db = sqlite::new();
    db.delete_list(list);
}

pub fn find_list(title_or_id: &String) -> Option<models::List> {
    let list = find_list_by_title(title_or_id);
    if list.is_some() {
        return list;
    }

    return find_list_by_id(title_or_id);
}

pub fn find_list_by_title(title: &String) -> Option<models::List> {
    let db = sqlite::new();
    return db.find_list_by_title(title);
}

pub fn find_list_by_id(id: &String) -> Option<models::List> {
    let db = sqlite::new();
    return db.find_list_by_id(id);
}

pub fn get_items(list_id: &String) -> Vec<models::Item> {
    let db = sqlite::new();
    return db.get_items(list_id);
}

pub fn create_item(list_id: &String, title: &String, description: &String) {
    let db = sqlite::new();
    db.create_item(list_id, title, description);
}

pub fn update_item(list_id: &String, item: &models::Item) {
    let db = sqlite::new();
    db.update_item(list_id, item);
}

pub fn delete_item(list_id: &String, item: &models::Item) {
    let db = sqlite::new();
    db.delete_item(list_id, item);
}

pub fn find_item(list_id: &String, id: &String) -> Option<models::Item> {
    let db = sqlite::new();
    return db.get_item(list_id, id);
}
