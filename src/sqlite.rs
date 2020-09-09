use crate::{config, log, models, utils};
use rusqlite::{params, Connection, Result, NO_PARAMS};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

const DB_VERSION: i16 = 2;

fn get_database_path(name: &str) -> Option<PathBuf> {
    if let Some(data_dir) = config::get_data_dir() {
        return Some(data_dir.join(name));
    }
    return None;
}

pub fn new() -> rusqlite::Connection {
    let db_path_buf = get_database_path("db.sqlite").expect("Failed to get database path");
    let db_path = Path::new(&db_path_buf);
    let new_database = !db_path.exists();

    let mut conn = Connection::open(&db_path).expect("Failed to open db");

    if new_database {
        create_database(&conn);
        if let Err(e) = migrate_database(&mut conn, 0) {
            println!("Failed to migrate database from version 0");
            println!("{}", e);
            std::process::exit(1);
        }
    } else {
        backup_database();
        match conn.query_row("PRAGMA user_version", NO_PARAMS, |row| Ok(row.get(0)?)) {
            Ok(db_version) => {
                if let Err(e) = migrate_database(&mut conn, db_version) {
                    println!(
                        "Failed to migrate database from version {} to {}",
                        db_version, DB_VERSION
                    );
                    println!("{}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                println!("Failed to read db version: {}", e);
            }
        }
    }

    return conn;
}

fn create_database(conn: &rusqlite::Connection) {
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

fn backup_database() {
    let db_path_buf = get_database_path("db.sqlite").expect("Failed to get database path");
    let db_bak_path_buf = get_database_path("db.sqlite.bak").expect("Failed to get database path");

    std::fs::copy(db_path_buf, db_bak_path_buf).expect("Failed to backup database file");
}

fn migrate_database(conn: &mut rusqlite::Connection, from: i16) -> Result<()> {
    if DB_VERSION <= from {
        return Ok(());
    }

    let now = utils::now();

    let tx = conn.transaction().expect("Failed to start transaction");
    for version in (from + 1)..DB_VERSION + 1 {
        match version {
            1 => {
                log::println(format!("Migrating to db version 1"));
                tx.execute(
                    "ALTER TABLE items ADD COLUMN state INTEGER NOT NULL DEFAULT 0",
                    NO_PARAMS,
                )?;
            }
            2 => {
                log::println(format!("Migrating to db version 2"));
                let sql_statements = vec![
                    "ALTER TABLE config ADD COLUMN last_server_sync BIGINT",
                    "ALTER TABLE config ADD COLUMN last_local_sync BIGINT",
                    "ALTER TABLE config ADD COLUMN next_list_id INTEGER",
                    "CREATE TABLE history (
                        id VARCHAR(36),
                        command TEXT,
                        state BLOB,
                        created BIGINT
                    )",
                ];

                for s in sql_statements.iter() {
                    tx.execute(s, NO_PARAMS)?;
                }

                tx.execute(
                    "CREATE TABLE list_update (
                        uuid VARCHAR(36),
                        id INTEGER,
                        title TEXT,
                        description TEXT,
                        created BIGINT,
                        modified BIGINT,
                        next_item_id INTEGER NOT NULL DEFAULT 1,
                        PRIMARY KEY (uuid)
                    )",
                    NO_PARAMS,
                )?;

                let mut stmt = tx.prepare("SELECT id, title, description FROM lists")?;
                let rows = stmt.query_map(NO_PARAMS, |row| {
                    let desc = match row.get::<_, String>(2) {
                        Ok(desc) => desc,
                        _ => String::from(""),
                    };
                    Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?, desc))
                })?;

                let mut list_id_uuid_map = HashMap::new();
                let mut largest_list_id: i32 = 0;
                for row in rows {
                    let uuid = Uuid::new_v4();
                    let (id, title, desc) = row.unwrap();
                    list_id_uuid_map.insert(id, uuid);
                    let uuid_str = uuid.to_hyphenated().to_string();
                    if largest_list_id < id {
                        largest_list_id = id;
                    }

                    tx.execute(
                        "INSERT INTO list_update (uuid, id, title, description, created, modified)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                        params![uuid_str, id, title, desc, now],
                    )?;
                }

                set_next_list_id(&tx, largest_list_id + 1)?;

                tx.execute("DROP TABLE lists", NO_PARAMS)?;
                tx.execute("ALTER TABLE list_update RENAME TO lists", NO_PARAMS)?;

                match tx.query_row(
                    "SELECT current_list FROM config WHERE id = 0",
                    NO_PARAMS,
                    |row| Ok(row.get::<_, String>(0)?),
                ) {
                    Ok(current) => match tx.query_row(
                        "SELECT uuid FROM lists WHERE id = ?1",
                        params![current],
                        |row| Ok(row.get::<_, String>(0)?),
                    ) {
                        Ok(uuid) => {
                            tx.execute(
                                "UPDATE config SET current_list = ?1 WHERE id = 0",
                                params![uuid],
                            )?;
                        }
                        Err(e) => {
                            log::println(format!("error: {}", e));
                        }
                    },
                    Err(e) => {
                        log::println(format!("error: {}", e));
                    }
                };

                tx.execute(
                    "CREATE TABLE items_update (
                        uuid VARCHAR(36),
                        id INTEGER,
                        title TEXT,
                        description TEXT,
                        state INTEGER,
                        created BIGINT,
                        modified BIGINT,
                        list_uuid VARCHAR(36) REFERENCES lists(uuid),
                        PRIMARY KEY (uuid)
                    )",
                    NO_PARAMS,
                )?;
                let mut stmt =
                    tx.prepare("SELECT id, title, description, state, list_id FROM items")?;
                let rows = stmt.query_map(NO_PARAMS, |row| {
                    let desc = match row.get::<_, String>(2) {
                        Ok(desc) => desc,
                        _ => String::from(""),
                    };
                    Ok((
                        row.get::<_, i32>(0)?,
                        row.get::<_, String>(1)?,
                        desc,
                        row.get::<_, i32>(3)?,
                        row.get::<_, i32>(4)?,
                    ))
                })?;

                let mut largest_list_item_id = HashMap::new();
                for row in rows {
                    let uuid = Uuid::new_v4();
                    let (id, title, desc, state, list_id) = row.unwrap();
                    let list_uuid = list_id_uuid_map
                        .get(&list_id)
                        .unwrap()
                        .to_hyphenated()
                        .to_string();
                    let uuid_str = uuid.to_hyphenated().to_string();

                    if let Some(largest) = largest_list_item_id.get(&list_uuid) {
                        if *largest < id {
                            largest_list_item_id.insert(list_uuid.clone(), id);
                        }
                    } else {
                        largest_list_item_id.insert(list_uuid.clone(), id);
                    }

                    tx.execute("
                        INSERT INTO items_update (uuid, id, title, description, state, created, modified, list_uuid)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7)",
                        params![uuid_str, id, title, desc, state, now, list_uuid]
                    )?;
                }

                for (uuid, largest_id) in largest_list_item_id.iter() {
                    tx.execute(
                        "UPDATE lists SET next_item_id = ?2 WHERE uuid = ?1",
                        params![uuid, largest_id + 1],
                    )?;
                }
                tx.execute("DROP TABLE items", NO_PARAMS)?;
                tx.execute("ALTER TABLE items_update RENAME TO items", NO_PARAMS)?;
            }
            _ => {}
        }
    }

    tx.execute(
        format!("PRAGMA user_version = {}", DB_VERSION).as_str(),
        NO_PARAMS,
    )?;

    tx.commit()?;

    Ok(())
}

fn row_to_list(row: &rusqlite::Row) -> Result<models::List> {
    Ok(models::List {
        uuid: Uuid::parse_str(row.get::<_, String>(0).unwrap().as_str()).unwrap(),
        id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        created: row.get(4)?,
        modified: row.get(5)?,
        next_item_id: row.get(6)?,
    })
}

fn row_to_item(row: &rusqlite::Row) -> Result<models::Item> {
    Ok(models::Item {
        uuid: Uuid::parse_str(row.get::<_, String>(0).unwrap().as_str()).unwrap(),
        id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        state: row.get(4)?,
        created: row.get(5)?,
        modified: row.get(6)?,
        list_uuid: Uuid::parse_str(row.get::<_, String>(7).unwrap().as_str()).unwrap(),
    })
}

pub fn transaction<F: FnMut(&rusqlite::Transaction) -> Result<()>>(
    conn: &mut Connection,
    mut f: F,
) -> Result<()> {
    let tx = conn.transaction()?;
    f(&tx)?;
    tx.commit()?;
    Ok(())
}

pub fn get_lists(conn: &Connection) -> Result<Vec<models::List>> {
    let mut stmt = conn.prepare(
        "SELECT uuid, id, title, description, created, modified, next_item_id
                FROM lists
                ORDER BY created",
    )?;

    let list_iter = stmt.query_map(NO_PARAMS, |row| row_to_list(row))?;
    let mut lists = Vec::new();
    for l in list_iter {
        lists.push(l.unwrap());
    }

    Ok(lists)
}

pub fn get_next_list_id(conn: &Connection) -> Result<i32> {
    return conn.query_row(
        "SELECT next_list_id FROM config WHERE id = 0",
        NO_PARAMS,
        |row| Ok(row.get(0)?),
    );
}

pub fn set_next_list_id(conn: &Connection, id: i32) -> Result<()> {
    conn.execute(
        "UPDATE config SET next_list_id = ?1 WHERE id = 0",
        params![id],
    )?;
    Ok(())
}

pub fn get_current_list(conn: &Connection) -> Result<Uuid> {
    conn.query_row(
        "SELECT current_list FROM config WHERE id = 0",
        NO_PARAMS,
        |row| {
            let uuid_str = row.get::<_, String>(0)?;
            if uuid_str == "" {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            Ok(Uuid::parse_str(uuid_str.as_str()).unwrap())
        },
    )
}

pub fn set_current_list(conn: &Connection, list_uuid: Option<&Uuid>) -> Result<()> {
    let to_set = match list_uuid {
        Some(uuid) => uuid.to_hyphenated().to_string(),
        None => String::new(),
    };

    conn.execute(
        "UPDATE config SET current_list = ?1 WHERE id = 0",
        params![to_set],
    )?;
    Ok(())
}

pub fn create_list(conn: &Connection, list: &models::List) -> Result<()> {
    conn.execute(
        "INSERT INTO lists (uuid, id, title, description, created, modified, next_item_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            list.uuid.to_hyphenated().to_string(),
            list.id,
            list.title,
            list.description,
            list.created,
            list.modified,
            list.next_item_id
        ],
    )?;
    Ok(())
}

pub fn update_list(conn: &Connection, list: &models::List) -> Result<()> {
    conn.execute(
        "UPDATE lists
            SET title = ?2, description = ?3, modified = ?4, next_item_id = ?5
            WHERE uuid = ?1",
        params![
            list.uuid.to_hyphenated().to_string(),
            list.title,
            list.description,
            list.modified,
            list.next_item_id,
        ],
    )?;
    Ok(())
}

pub fn delete_list(conn: &Connection, list: &models::List) -> Result<()> {
    conn.execute(
        "DELETE FROM lists WHERE uuid = ?1",
        params![list.uuid.to_hyphenated().to_string()],
    )?;
    Ok(())
}

pub fn find_list_by_uuid(conn: &Connection, uuid: &Uuid) -> Result<models::List> {
    conn.query_row(
        "SELECT uuid, id, title, description, created, modified, next_item_id
            FROM lists
            WHERE uuid = (?1)",
        params![uuid.to_hyphenated().to_string()],
        |row| row_to_list(row),
    )
}

pub fn find_list_by_id(conn: &Connection, id: &String) -> Result<models::List> {
    conn.query_row(
        "SELECT uuid, id, title, description, created, modified, next_item_id
            FROM lists
            WHERE id = (?1)",
        params![id],
        |row| row_to_list(row),
    )
}

pub fn get_items(conn: &Connection, list_uuid: &Uuid) -> Result<Vec<models::Item>> {
    let mut stmt = conn.prepare(
        "SELECT uuid, id, title, description, state, created, modified, list_uuid
                FROM items
                WHERE list_uuid = ?1",
    )?;

    let iter = stmt.query_map(params![list_uuid.to_hyphenated().to_string()], |row| {
        row_to_item(row)
    })?;

    let mut items = Vec::new();
    for v in iter {
        items.push(v.unwrap());
    }
    return Ok(items);
}

pub fn get_item(conn: &Connection, list_uuid: &Uuid, item_id: &String) -> Result<models::Item> {
    conn.query_row(
        "SELECT uuid, id, title, description, state, created, modified, list_uuid
            FROM items
            WHERE list_uuid = (?1)
                AND id = (?2)",
        params![list_uuid.to_hyphenated().to_string(), item_id],
        |row| row_to_item(row),
    )
}

pub fn create_item(conn: &Connection, item: &models::Item) -> Result<()> {
    conn.execute(
        "INSERT INTO items (uuid, id, title, description, state, created, modified, list_uuid)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            item.uuid.to_hyphenated().to_string(),
            item.id,
            item.title,
            item.description,
            item.state,
            item.created,
            item.modified,
            item.list_uuid.to_hyphenated().to_string()
        ],
    )?;
    Ok(())
}

pub fn update_item(conn: &Connection, item: &models::Item) -> Result<()> {
    conn.execute(
        "UPDATE items
            SET title = ?3, description = ?4, state = ?5
            WHERE list_uuid = ?1 AND uuid = ?2",
        params![
            item.list_uuid.to_hyphenated().to_string(),
            item.uuid.to_hyphenated().to_string(),
            item.title,
            item.description,
            item.state
        ],
    )?;
    Ok(())
}

pub fn delete_item(conn: &Connection, item: &models::Item) -> Result<()> {
    conn.execute(
        "DELETE FROM items
            WHERE list_uuid = ?1
                AND uuid = ?2",
        params![
            item.list_uuid.to_hyphenated().to_string(),
            item.uuid.to_hyphenated().to_string()
        ],
    )?;
    Ok(())
}
