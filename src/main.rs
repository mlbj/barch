mod db;
mod cli;
mod reference;
mod service;
mod bibtex;

use clap::Parser;
use cli::{Cli, Commands};
use std::io::{self, Read};
use directories::ProjectDirs;
use std::path::PathBuf;
use std::fs;

fn get_db_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("", "", "bark")
        .expect("Could not determine data directory");

    let data_dir = proj_dirs.data_dir();

    std::fs::create_dir_all(data_dir).unwrap();

    data_dir.join("library.db")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let db_path = get_db_path();
    let conn = db::init_db(db_path.to_str().unwrap())?;

    match cli.command {
        Commands::Add => {
            println!("Paste BibTeX, Ctrl+D to finish:\n");

            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;

            let id = service::add_reference(&conn, &input)?;
            println!("Saved as {}", id);
        }

        Commands::List { tag } => {
            let refs = service::list_references(&conn, tag.as_deref())?;
            
            // Compute column widths
            let max_key = refs.iter().map(|r| r.key.len()).max().unwrap_or(0);
            let max_id = 8;
            
            for r in refs {
                let short_id = &r.id[..8];

                let title = r.title.unwrap_or_else(|| "<no title>".to_string());

                let tag_str = if r.tags.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", r.tags.join(", "))
                };

                println!(
                    "{:width_id$}  {:width_key$}  {}{}",
                    short_id,
                    r.key,
                    title,
                    tag_str,
                    width_id = 8,
                    width_key = max_key,
                );
            }
        }

        Commands::Show { input } => {
            let id = db::resolve_reference(&conn, &input)?;
            let bib = db::get_reference(&conn, &id)?;
            println!("{}", bib);
        }

        Commands::Export { tag } => {
            let filename = "references.bib";
            let mut content = String::new();
            
            // Fill content
            let refs = service::list_references(&conn, tag.as_deref())?;
            for r in refs {
                let bib = db::get_reference(&conn, &r.id)?;
                content.push_str(&bib);
                content.push_str("\n\n");
            }

            // Save file
            fs::write(filename, content)?;
        }

        Commands::Import { filename } => {
            let result = service::import_bibtex(&conn, &filename)?;

            println!(
                "Imported: {} | Skipped: {}",
                result.added, result.skipped
            );
        }

        Commands::Tag { input, tag } => {
            let id = db::resolve_reference(&conn, &input)?;
            db::add_tag_to_reference(&conn, &id, &tag)?;
        }
    }

    Ok(())
}
