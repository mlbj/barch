use rusqlite::{Connection, Result};
use uuid::Uuid;

pub fn init_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS refs (
            id TEXT PRIMARY KEY,
            bibtex TEXT NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS reference_files (
            reference_id TEXT,
            file_id TEXT,
            PRIMARY KEY (reference_id, file_id)
        );
        "
    )?;

    Ok(conn)
}

pub fn add_reference(conn: &Connection, bibtex: &str) -> Result<String> {
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO refs (id, bibtex) VALUES (?1, ?2)",
        (&id, bibtex),
    )?;

    Ok(id)
}

pub fn list_references(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, substr(bibtex, 1, 60) FROM refs"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }

    Ok(result)
}

pub fn get_reference(conn: &Connection, id: &str) -> Result<String> {
    conn.query_row(
        "SELECT bibtex FROM refs WHERE id = ?1",
        [id],
        |row| row.get::<_, String>(0),
    )
}
