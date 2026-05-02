pub mod service;
pub mod reference;

pub mod db;
pub mod bibtex;

// Re-export API
pub use service::{
    add_reference,
    list_references,
    import_bibtex,
    resolve_reference,
    get_reference,
    ImportResult,
};

pub use reference::Reference;

use rusqlite::Connection;

pub struct Bark {
    conn: Connection,
}

impl Bark {
    pub fn new(path: &str) -> rusqlite::Result<Self> {
        let conn = db::init_db(path)?;
        Ok(Self { conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
