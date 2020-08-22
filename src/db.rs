use crate::models;

pub trait Database {
    fn get_lists(&self) -> Vec<models::List>;

    fn create_list(&self, title: &String);
    fn get_list(&self, title: &String) -> Option<models::List>;
}

pub mod sqlite {
    use crate::models;
    use rusqlite::{params, Connection, NO_PARAMS};
    use std::path::Path;

    pub struct SQLiteDatabase {
        conn: rusqlite::Connection,
    }

    pub fn new() -> SQLiteDatabase {
        let path = "./db.sqlite";
        let new_database = !Path::new(path).exists();

        let conn = Connection::open(&path).expect("Failed to open db");
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

        fn get_list(&self, title: &String) -> Option<models::List> {
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
                Err(e) => {
                    println!("Error: {:?}", e);
                    None
                }
            }
        }

        fn create_list(&self, title: &String) {
            self.conn
                .execute("INSERT INTO lists (title) VALUES (?1)", params![title])
                .expect("Failed to create list");
        }
    }
}

pub fn get_lists() -> Vec<models::List> {
    let db = sqlite::new();
    return db.get_lists();
}
