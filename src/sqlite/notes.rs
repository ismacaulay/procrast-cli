use crate::{
    models::{self},
    utils::Result,
};
use rusqlite::{params, Connection};
use uuid::Uuid;

pub fn create(conn: &Connection, note: &models::Note) -> Result<()> {
    if let Err(e) = conn.execute(
        "INSERT INTO notes (uuid, id, title, body, created, modified, list_uuid)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            note.uuid.to_hyphenated().to_string(),
            note.id,
            note.title,
            note.body,
            note.created,
            note.modified,
            note.list_uuid.to_hyphenated().to_string()
        ],
    ) {
        return Err(e.to_string());
    }

    Ok(())
}

pub fn get(conn: &Connection, list_uuid: &Uuid) -> Result<Vec<models::Note>> {
    let mut stmt = match conn.prepare(
        "SELECT uuid, id, title, body, created, modified, list_uuid
                FROM notes
                WHERE list_uuid = ?1
                ORDER BY
                    state ASC,
                    id ASC
                ",
    ) {
        Ok(stmt) => stmt,
        Err(e) => return Err(e.to_string()),
    };

    let iter = match stmt.query_map(params![list_uuid.to_hyphenated().to_string()], |row| {
        row_to_note(row)
    }) {
        Ok(iter) => iter,
        Err(e) => return Err(e.to_string()),
    };

    let mut notes = Vec::new();
    for v in iter {
        notes.push(v.unwrap());
    }
    return Ok(notes);
}

pub fn update(conn: &Connection, note: &models::Note) -> Result<()> {
    if let Err(e) = conn.execute(
        "UPDATE notes
            SET title = ?3, body = ?4
            WHERE list_uuid = ?1 AND uuid = ?2",
        params![
            note.list_uuid.to_hyphenated().to_string(),
            note.uuid.to_hyphenated().to_string(),
            note.title,
            note.body,
        ],
    ) {
        return Err(e.to_string());
    }

    Ok(())
}

pub fn delete(conn: &Connection, note: &models::Note) -> Result<()> {
    if let Err(e) = conn.execute(
        "DELETE FROM notes
            WHERE list_uuid = ?1
                AND uuid = ?2",
        params![
            note.list_uuid.to_hyphenated().to_string(),
            note.uuid.to_hyphenated().to_string()
        ],
    ) {
        return Err(e.to_string());
    }

    Ok(())
}

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<models::Note> {
    Ok(models::Note {
        uuid: Uuid::parse_str(row.get::<_, String>(0).unwrap().as_str()).unwrap(),
        id: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        created: row.get(5)?,
        modified: row.get(6)?,
        list_uuid: Uuid::parse_str(row.get::<_, String>(7).unwrap().as_str()).unwrap(),
    })
}
