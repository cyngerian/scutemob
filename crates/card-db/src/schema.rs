//! SQLite schema definition and table creation.
//!
//! Schema follows architecture doc Section 5.2.

use rusqlite::Connection;

use crate::Result;

/// Creates all tables if they don't already exist.
pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
            oracle_id TEXT NOT NULL,
            name TEXT NOT NULL,
            mana_cost TEXT,
            cmc REAL NOT NULL,
            type_line TEXT NOT NULL,
            oracle_text TEXT,
            power TEXT,
            toughness TEXT,
            loyalty TEXT,
            colors TEXT,
            color_identity TEXT,
            keywords TEXT,
            legalities TEXT,
            set_code TEXT NOT NULL,
            collector_number TEXT NOT NULL,
            rarity TEXT,
            layout TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_cards_oracle_id ON cards(oracle_id);
        CREATE INDEX IF NOT EXISTS idx_cards_name ON cards(name);

        CREATE TABLE IF NOT EXISTS card_faces (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id TEXT NOT NULL,
            face_index INTEGER NOT NULL,
            name TEXT NOT NULL,
            mana_cost TEXT,
            type_line TEXT NOT NULL,
            oracle_text TEXT,
            power TEXT,
            toughness TEXT,
            colors TEXT,
            FOREIGN KEY (card_id) REFERENCES cards(id)
        );

        CREATE INDEX IF NOT EXISTS idx_card_faces_card_id ON card_faces(card_id);

        CREATE TABLE IF NOT EXISTS rulings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            oracle_id TEXT NOT NULL,
            published_at TEXT NOT NULL,
            comment TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_rulings_oracle_id ON rulings(oracle_id);

        CREATE TABLE IF NOT EXISTS card_definitions (
            oracle_id TEXT PRIMARY KEY,
            definition_json TEXT NOT NULL,
            definition_version INTEGER NOT NULL,
            validated INTEGER DEFAULT 0,
            validation_notes TEXT
        );
        ",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();

        // Verify tables exist by querying sqlite_master
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"cards".to_string()));
        assert!(tables.contains(&"card_faces".to_string()));
        assert!(tables.contains(&"rulings".to_string()));
        assert!(tables.contains(&"card_definitions".to_string()));
    }

    #[test]
    fn test_schema_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        // Running again should not error
        create_tables(&conn).unwrap();
    }
}
