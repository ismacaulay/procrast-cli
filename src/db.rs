use crate::models;

pub trait Database {
    fn get_lists(&self) -> Vec<models::List>;

    fn create_list(&self, title: &String, description: &String);
    fn update_list(&self, list: &models::List);
    fn delete_list(&self, list: &models::List);
    fn find_list_by_title(&self, title: &String) -> Option<models::List>;
    fn find_list_by_id(&self, id: &String) -> Option<models::List>;
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
    }
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

pub fn find_list_by_title(title: &String) -> Option<models::List> {
    let db = sqlite::new();
    return db.find_list_by_title(title);
}

pub fn find_list_by_id(id: &String) -> Option<models::List> {
    let db = sqlite::new();
    return db.find_list_by_id(id);
}
