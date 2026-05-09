use rusqlite::{Connection, Result};
use uuid::Uuid;
use std::fs;

use crate::db;
use crate::reference::Reference;
use crate::bibtex::{parse_bibtex_header, extract_field_bibtex, split_bibtex_entries};

use serde::{Serialize, Deserialize};

pub struct ImportResult {
    pub added: usize,
    pub skipped: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ExportV1 {
    pub version: u32,
    pub references: Vec<ExportReference>,
}

#[derive(Serialize, Deserialize)]
pub struct ExportReference {
    pub id: String,
    pub bibtex: String,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ExportContent>,
}

#[derive(Serialize, Deserialize)]
pub struct ExportContent {
    pub kind: String,
    pub location: String,
}

pub fn add_reference(conn: &Connection, bibtex: &str) -> Result<String> {
    let id = Uuid::new_v4().to_string();

    let (entry_type, entry_key) =
        parse_bibtex_header(bibtex)
            .ok_or_else(|| rusqlite::Error::InvalidQuery)?;

    let title = extract_field_bibtex(bibtex, "title");

    db::insert_reference(
        conn,
        &id,
        bibtex,
        &entry_type,
        &entry_key,
        title.as_deref(),
    )?;

    Ok(id)
}

pub fn remove_reference(
    conn: &Connection,
    input: &str,
) -> Result<()> {
    let id = db::resolve_reference(conn, input)?;
    db::remove_reference(conn, &id)
}


pub fn list_references(
    conn: &Connection,
    tag: Option<&str>,
) -> Result<Vec<Reference>> {
    let raw = db::list_references(conn, tag)?;

    let mut result = Vec::new();

    for (id, key, title, tags) in raw {
        let tags_vec = tags
            .map(|t| t.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_else(Vec::new);

        result.push(Reference {
            id,
            key,
            title,
            tags: tags_vec,
        });
    }

    Ok(result)
}

pub fn import_bibtex(conn: &Connection, path: &str) -> Result<ImportResult> {
    let content = fs::read_to_string(path)
        .map_err(|_| rusqlite::Error::InvalidQuery)?;

    let entries = split_bibtex_entries(&content);

    let mut added = 0;
    let mut skipped = 0;

    for entry in entries {
        let (_ty, key) = match parse_bibtex_header(&entry) {
            Some(v) => v,
            None => {
                skipped += 1;
                continue;
            }
        };

        match add_reference(conn, &entry) {
            Ok(_) => added += 1,
            Err(_) => skipped += 1,
        }
    }

    Ok(ImportResult { added, skipped })
}

pub fn resolve_reference(conn: &Connection, input: &str) -> Result<String> {
    db::resolve_reference(conn, input)
}

pub fn get_reference(conn: &Connection, input: &str) -> Result<String> {
    let id = db::resolve_reference(conn, input)?;
    db::get_reference(conn, &id)
}

pub fn add_tag(conn: &Connection, input: &str, tag: &str) -> Result<()> {
    let id = db::resolve_reference(conn, input)?;
    db::insert_tag(conn, &id, tag)
}

pub fn export_references(
    conn: &Connection,
    tag: Option<&str>,
) -> Result<String> {
    let references = list_references(conn, tag)?;

    let mut content = String::new();
    for r in references {
        let bib = db::get_reference(conn, &r.id)?;
        content.push_str(&bib);
        content.push_str("\n\n");
    }

    Ok(content)
}

fn infer_kind(location: &str) -> &str {
    if location.starts_with("http://") || location.starts_with("https://") {
        "url"
    } else if location.contains('@') && location.contains(':') {
        // Simple ssh heuristic for now
        "ssh"
    } else {
        "local"
    }
}

pub fn add_content(
    conn: &Connection,
    input: &str,
    location: &str,
) -> Result<()> {
    let id = db::resolve_reference(conn, input)?;
    let kind = infer_kind(location);

    db::insert_content(conn, &id, kind, location)
}

pub fn get_content(
    conn: &Connection,
    input: &str,
) -> Result<(String, String)> {
    let id = db::resolve_reference(conn, input)?;
    db::get_content(conn, &id)
}

pub fn export_toml(conn: &Connection) -> Result<String> {
    let raw_references = db::list_references(conn, None)?;

    let mut references = Vec::new();

    for (id, _key, _title, _tags_str) in raw_references {
        let bibtex = db::get_reference(conn, &id)?;
        let tags = db::get_tags_for_reference(conn, &id)?;

        let content = match db::get_content(conn, &id) {
            Ok((kind, location)) => Some(ExportContent { kind, location }),
            Err(_) => None,
        };

        references.push(ExportReference {
            id,
            bibtex,
            tags,
            content,
        });
    }

    let export = ExportV1 {
        version: 1,
        references,
    };

    Ok(toml::to_string_pretty(&export).unwrap())
}

pub fn import_toml(conn: &Connection, path: &str) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .map_err(|_| rusqlite::Error::InvalidQuery)?;

    let data: ExportV1 = toml::from_str(&content)
        .map_err(|_| rusqlite::Error::InvalidQuery)?;

    if data.version != 1 {
        return Err(rusqlite::Error::InvalidQuery);
    }

    conn.execute_batch(
        "
        DELETE FROM reference_tags;
        DELETE FROM tags;
        DELETE FROM content;
        DELETE FROM refs;
        "
    )?;

    for r in data.references {
        let (entry_type, entry_key) =
            parse_bibtex_header(&r.bibtex)
                .ok_or(rusqlite::Error::InvalidQuery)?;

        let title = extract_field_bibtex(&r.bibtex, "title");

        db::insert_reference(
            conn,
            &r.id,
            &r.bibtex,
            &entry_type,
            &entry_key,
            title.as_deref(),
        )?;

        // tags
        for tag in r.tags {
            db::insert_tag(conn, &r.id, &tag)?;
        }

        // content (0 or 1)
        if let Some(c) = r.content {
            db::insert_content(conn, &r.id, &c.kind, &c.location)?;
        }
    }

    Ok(())
}
