use crate::utils::Result;
use rusqlite::{self, NO_PARAMS};

fn v3(tx: &rusqlite::Transaction) -> Result<()> {
    let sql_statements = vec![
        "CREATE TABLE notes (
            uuid VARCHAR(36),
            id INTEGER,
            title TEXT,
            body TEXT,
            created BIGINT,
            modified BIGINT,
            list_uuid VARCHAR(36) REFERENCES lists(uuid),
            PRIMARY KEY (uuid)
        )",
        "ALTER TABLE lists ADD COLUMN next_notes_id INTEGER NOT NULL DEFAULT 1",
    ];

    for s in sql_statements.iter() {
        if let Err(e) = tx.execute(s, NO_PARAMS) {
            return Err(format!("Failed to execute query: {}", e.to_string()));
        }
    }
    Ok(())
}
