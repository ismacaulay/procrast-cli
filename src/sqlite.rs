use crate::{
    config, log,
    models::{self, CMD_ITEM_CREATE, CMD_LIST_CREATE},
    utils,
};
use rusqlite::{params, Connection, NO_PARAMS};
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

fn migrate_database(conn: &mut rusqlite::Connection, from: i16) -> utils::Result<()> {
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
                )
                .expect("Failed to add state to items");
            }
            2 => {
                log::println(format!("Migrating to db version 2"));
                let sql_statements = vec![
                    "ALTER TABLE config ADD COLUMN last_server_sync BIGINT",
                    "ALTER TABLE config ADD COLUMN last_local_sync BIGINT",
                    "ALTER TABLE config ADD COLUMN next_list_id INTEGER",
                    "CREATE TABLE history (
                        uuid VARCHAR(36),
                        command TEXT,
                        state BLOB,
                        created BIGINT,
                        synced INT
                    )",
                ];

                for s in sql_statements.iter() {
                    tx.execute(s, NO_PARAMS).expect("Failed to execute query");
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
                )
                .expect("Failed to create list table");

                let mut stmt = tx
                    .prepare("SELECT id, title, description FROM lists")
                    .expect("Failed to prepare list quest");
                let rows = stmt
                    .query_map(NO_PARAMS, |row| {
                        let desc = match row.get::<_, String>(2) {
                            Ok(desc) => desc,
                            _ => String::from(""),
                        };
                        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?, desc))
                    })
                    .expect("Failed to get list rows");

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
                    )
                    .expect("Failed to insert lists");

                    let state = utils::encode_history_state(&models::CmdListState {
                        uuid: uuid,
                        title: title,
                        description: desc,
                        created: now,
                        modified: now,
                    })?;
                    tx.execute(
                        "INSERT INTO history (uuid, command, state, created, synced)
                        VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![
                            Uuid::new_v4().to_hyphenated().to_string(),
                            CMD_LIST_CREATE,
                            state,
                            now,
                            false
                        ],
                    )
                    .expect("Failed to create history for list");
                }

                set_next_list_id(&tx, largest_list_id + 1)?;

                tx.execute("DROP TABLE lists", NO_PARAMS)
                    .expect("Failed to drop old lists");
                tx.execute("ALTER TABLE list_update RENAME TO lists", NO_PARAMS)
                    .expect("Failed to update lists");

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
                            )
                            .expect("Failed to set current list");
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
                )
                .expect("Failed to create new items table");
                let mut stmt = tx
                    .prepare("SELECT id, title, description, state, list_id FROM items")
                    .expect("Failed to prepare items query");
                let rows = stmt
                    .query_map(NO_PARAMS, |row| {
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
                    })
                    .expect("Failed to get items rows");

                let mut largest_list_item_id = HashMap::new();
                for row in rows {
                    let uuid = Uuid::new_v4();
                    let (id, title, desc, state, list_id) = row.unwrap();
                    let list_uuid = list_id_uuid_map.get(&list_id).unwrap();
                    let list_uuid_str = list_uuid.to_hyphenated().to_string();
                    let uuid_str = uuid.to_hyphenated().to_string();

                    if let Some(largest) = largest_list_item_id.get(&list_uuid_str) {
                        if *largest < id {
                            largest_list_item_id.insert(list_uuid_str.clone(), id);
                        }
                    } else {
                        largest_list_item_id.insert(list_uuid_str.clone(), id);
                    }

                    tx.execute("
                        INSERT INTO items_update (uuid, id, title, description, state, created, modified, list_uuid)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7)",
                        params![uuid_str, id, title, desc, state, now, list_uuid_str]
                    ).expect("Failed to add item to items table");

                    let state = utils::encode_history_state(&models::CmdItemState {
                        uuid: uuid,
                        title: title,
                        description: desc,
                        state: state as i8,
                        created: now,
                        modified: now,
                        list_uuid: *list_uuid,
                    })?;
                    tx.execute(
                        "INSERT INTO history (uuid, command, state, created, synced)
                        VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![
                            Uuid::new_v4().to_hyphenated().to_string(),
                            CMD_ITEM_CREATE,
                            state,
                            now,
                            false
                        ],
                    )
                    .expect("Failed to insert item history");
                }

                for (uuid, largest_id) in largest_list_item_id.iter() {
                    tx.execute(
                        "UPDATE lists SET next_item_id = ?2 WHERE uuid = ?1",
                        params![uuid, largest_id + 1],
                    )
                    .expect("Failed to update lists next_item_id");
                }
                tx.execute("DROP TABLE items", NO_PARAMS)
                    .expect("Failed to remove old items table");
                tx.execute("ALTER TABLE items_update RENAME TO items", NO_PARAMS)
                    .expect("Failed to updated items table");
            }
            _ => {}
        }
    }

    tx.execute(
        format!("PRAGMA user_version = {}", DB_VERSION).as_str(),
        NO_PARAMS,
    )
    .expect("Failed to update user_version");

    tx.commit().expect("Failed to commit transaction");

    Ok(())
}

fn row_to_list(row: &rusqlite::Row) -> rusqlite::Result<models::List> {
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

fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<models::Item> {
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

fn row_to_history(row: &rusqlite::Row) -> rusqlite::Result<models::History> {
    Ok(models::History {
        uuid: Uuid::parse_str(row.get::<_, String>(0).unwrap().as_str()).unwrap(),
        command: row.get(1)?,
        state: row.get(2)?,
        timestamp: row.get(3)?,
        synced: row.get(4)?,
    })
}

pub fn transaction<F: FnMut(&rusqlite::Transaction) -> utils::Result<()>>(
    conn: &mut Connection,
    mut f: F,
) -> utils::Result<()> {
    let tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(e) => return Err(e.to_string()),
    };

    f(&tx)?;

    if let Err(e) = tx.commit() {
        return Err(e.to_string());
    }
    Ok(())
}

pub fn get_lists(conn: &Connection) -> utils::Result<Vec<models::List>> {
    let mut stmt = match conn.prepare(
        "SELECT uuid, id, title, description, created, modified, next_item_id
                FROM lists
                ORDER BY id",
    ) {
        Ok(stmt) => stmt,
        Err(e) => return Err(e.to_string()),
    };

    let list_iter = match stmt.query_map(NO_PARAMS, |row| row_to_list(row)) {
        Ok(iter) => iter,
        Err(e) => return Err(e.to_string()),
    };

    let mut lists = Vec::new();
    for l in list_iter {
        lists.push(l.unwrap());
    }

    Ok(lists)
}

pub fn get_next_list_id(conn: &Connection) -> utils::Result<i32> {
    match conn.query_row(
        "SELECT next_list_id FROM config WHERE id = 0",
        NO_PARAMS,
        |row| Ok(row.get(0)?),
    ) {
        Ok(id) => Ok(id),
        Err(e) => Err(e.to_string()),
    }
}

pub fn set_next_list_id(conn: &Connection, id: i32) -> utils::Result<()> {
    if let Err(e) = conn.execute(
        "UPDATE config SET next_list_id = ?1 WHERE id = 0",
        params![id],
    ) {
        return Err(e.to_string());
    }
    Ok(())
}

pub fn get_current_list(conn: &Connection) -> utils::Result<Uuid> {
    match conn.query_row(
        "SELECT current_list FROM config WHERE id = 0",
        NO_PARAMS,
        |row| {
            let uuid_str = row.get::<_, String>(0)?;
            if uuid_str == "" {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            Ok(Uuid::parse_str(uuid_str.as_str()).unwrap())
        },
    ) {
        Ok(uuid) => Ok(uuid),
        Err(e) => Err(e.to_string()),
    }
}

pub fn set_current_list(conn: &Connection, list_uuid: Option<&Uuid>) -> utils::Result<()> {
    let to_set = match list_uuid {
        Some(uuid) => uuid.to_hyphenated().to_string(),
        None => String::new(),
    };

    if let Err(e) = conn.execute(
        "UPDATE config SET current_list = ?1 WHERE id = 0",
        params![to_set],
    ) {
        return Err(e.to_string());
    }
    Ok(())
}

pub fn get_last_local_sync(conn: &Connection) -> utils::Result<i64> {
    match conn.query_row(
        "SELECT last_local_sync FROM config WHERE id = 0",
        NO_PARAMS,
        |row| Ok(row.get::<_, i64>(0)?),
    ) {
        Ok(t) => Ok(t),
        Err(e) => Err(e.to_string()),
    }
}

pub fn set_last_local_sync(conn: &Connection, ts: i64) -> utils::Result<()> {
    if let Err(e) = conn.execute(
        "UPDATE config SET last_local_sync = ?1 WHERE id = 0",
        params![ts],
    ) {
        return Err(e.to_string());
    }
    Ok(())
}

pub fn create_list(conn: &Connection, list: &models::List) -> utils::Result<()> {
    if let Err(e) = conn.execute(
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
    ) {
        return Err(e.to_string());
    }
    Ok(())
}

pub fn update_list(conn: &Connection, list: &models::List) -> utils::Result<()> {
    if let Err(e) = conn.execute(
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
    ) {
        return Err(e.to_string());
    }
    Ok(())
}

pub fn delete_list(conn: &Connection, list: &models::List) -> utils::Result<()> {
    if let Err(e) = conn.execute(
        "DELETE FROM lists WHERE uuid = ?1",
        params![list.uuid.to_hyphenated().to_string()],
    ) {
        return Err(e.to_string());
    }
    Ok(())
}

pub fn find_list_by_uuid(conn: &Connection, uuid: &Uuid) -> utils::Result<models::List> {
    match conn.query_row(
        "SELECT uuid, id, title, description, created, modified, next_item_id
            FROM lists
            WHERE uuid = (?1)",
        params![uuid.to_hyphenated().to_string()],
        |row| row_to_list(row),
    ) {
        Ok(list) => Ok(list),
        Err(e) => Err(e.to_string()),
    }
}

pub fn find_list_by_id(conn: &Connection, id: &String) -> utils::Result<models::List> {
    match conn.query_row(
        "SELECT uuid, id, title, description, created, modified, next_item_id
            FROM lists
            WHERE id = (?1)",
        params![id],
        |row| row_to_list(row),
    ) {
        Ok(list) => Ok(list),
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_items(conn: &Connection, list_uuid: &Uuid) -> utils::Result<Vec<models::Item>> {
    let mut stmt = match conn.prepare(
        "SELECT uuid, id, title, description, state, created, modified, list_uuid
                FROM items
                WHERE list_uuid = ?1",
    ) {
        Ok(stmt) => stmt,
        Err(e) => return Err(e.to_string()),
    };

    let iter = match stmt.query_map(params![list_uuid.to_hyphenated().to_string()], |row| {
        row_to_item(row)
    }) {
        Ok(iter) => iter,
        Err(e) => return Err(e.to_string()),
    };

    let mut items = Vec::new();
    for v in iter {
        items.push(v.unwrap());
    }
    return Ok(items);
}

pub fn get_incomplete_items(
    conn: &Connection,
    list_uuid: &Uuid,
) -> utils::Result<Vec<models::Item>> {
    let mut stmt = match conn.prepare(
        "SELECT uuid, id, title, description, state, created, modified, list_uuid
                FROM items
                WHERE list_uuid = ?1
                AND state = 0",
    ) {
        Ok(stmt) => stmt,
        Err(e) => return Err(e.to_string()),
    };

    let iter = match stmt.query_map(params![list_uuid.to_hyphenated().to_string()], |row| {
        row_to_item(row)
    }) {
        Ok(iter) => iter,
        Err(e) => return Err(e.to_string()),
    };

    let mut items = Vec::new();
    for v in iter {
        items.push(v.unwrap());
    }
    return Ok(items);
}

pub fn get_item(
    conn: &Connection,
    list_uuid: &Uuid,
    item_id: &String,
) -> utils::Result<models::Item> {
    match conn.query_row(
        "SELECT uuid, id, title, description, state, created, modified, list_uuid
            FROM items
            WHERE list_uuid = (?1)
                AND id = (?2)",
        params![list_uuid.to_hyphenated().to_string(), item_id],
        |row| row_to_item(row),
    ) {
        Ok(item) => Ok(item),
        Err(e) => Err(e.to_string()),
    }
}

pub fn find_item_by_uuid(conn: &Connection, item_uuid: &Uuid) -> utils::Result<models::Item> {
    match conn.query_row(
        "SELECT uuid, id, title, description, state, created, modified, list_uuid
            FROM items
            WHERE uuid = (?1)",
        params![item_uuid.to_hyphenated().to_string()],
        |row| row_to_item(row),
    ) {
        Ok(item) => Ok(item),
        Err(e) => Err(e.to_string()),
    }
}
pub fn create_item(conn: &Connection, item: &models::Item) -> utils::Result<()> {
    if let Err(e) = conn.execute(
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
    ) {
        return Err(e.to_string());
    }

    Ok(())
}

pub fn update_item(conn: &Connection, item: &models::Item) -> utils::Result<()> {
    if let Err(e) = conn.execute(
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
    ) {
        return Err(e.to_string());
    }

    Ok(())
}

pub fn delete_item(conn: &Connection, item: &models::Item) -> utils::Result<()> {
    if let Err(e) = conn.execute(
        "DELETE FROM items
            WHERE list_uuid = ?1
                AND uuid = ?2",
        params![
            item.list_uuid.to_hyphenated().to_string(),
            item.uuid.to_hyphenated().to_string()
        ],
    ) {
        return Err(e.to_string());
    }

    Ok(())
}

pub fn create_history(conn: &Connection, history: &models::History) -> utils::Result<()> {
    if let Err(e) = conn.execute(
        "INSERT INTO history (uuid, command, state, created, synced)
                VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            history.uuid.to_hyphenated().to_string(),
            history.command,
            history.state,
            history.timestamp,
            history.synced,
        ],
    ) {
        return Err(e.to_string());
    }

    Ok(())
}

pub fn find_history_by_uuid(
    conn: &Connection,
    uuid: &uuid::Uuid,
) -> utils::Result<models::History> {
    match conn.query_row(
        "SELECT uuid, command, state, created, synced
            FROM history
            WHERE uuid = (?1)",
        params![uuid.to_hyphenated().to_string()],
        |row| row_to_history(row),
    ) {
        Ok(history) => Ok(history),
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_history(conn: &Connection) -> utils::Result<Vec<models::History>> {
    let mut stmt = match conn.prepare(
        "SELECT uuid, command, state, created, synced
                FROM history
                ORDER BY created ASC",
    ) {
        Ok(stmt) => stmt,
        Err(e) => return Err(e.to_string()),
    };

    let iter = match stmt.query_map(NO_PARAMS, |row| row_to_history(row)) {
        Ok(iter) => iter,
        Err(e) => return Err(e.to_string()),
    };

    let mut history = Vec::new();
    for h in iter {
        history.push(h.unwrap());
    }

    Ok(history)
}

pub fn get_unsynced_history(conn: &Connection) -> utils::Result<Vec<models::History>> {
    let mut stmt = match conn.prepare(
        "SELECT uuid, command, state, created, synced
                FROM history
                WHERE synced = 0
                ORDER BY created ASC",
    ) {
        Ok(stmt) => stmt,
        Err(e) => return Err(e.to_string()),
    };

    let iter = match stmt.query_map(NO_PARAMS, |row| row_to_history(row)) {
        Ok(iter) => iter,
        Err(e) => return Err(e.to_string()),
    };

    let mut history = Vec::new();
    for h in iter {
        history.push(h.unwrap());
    }

    Ok(history)
}

pub fn update_history_synced(
    conn: &Connection,
    uuid: &uuid::Uuid,
    synced: bool,
) -> utils::Result<()> {
    if let Err(e) = conn.execute(
        "UPDATE history
            SET synced = ?2
            WHERE uuid = ?1",
        params![uuid.to_hyphenated().to_string(), synced],
    ) {
        return Err(e.to_string());
    }

    Ok(())
}
