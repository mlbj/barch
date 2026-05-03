use clap::{Parser, Subcommand};

use std::io::{self, Read};
use std::fs;
use std::env;
use std::path::PathBuf;

use bark_core::{service, Bark};

#[derive(Parser)]
#[command(name = "bark",
          version,
          about = "A minimal, headless reference manager for storing BibTex entries and associated files"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new reference (reads BibTeX from stdin)
    Add,

    /// List stored references
    List {
        /// Filter by tag
        #[arg(index = 1)]
        tag: Option<String>,
    },

    /// Show full BibTeX entry
    Show {
        /// Entry key, full id or short id
        input: String },

    /// Export references to reference.bib file
    Export {
        /// Filter by tag
        #[arg(index = 1)]
        tag: Option<String>,
    },

    /// Import references from a .bib file
    Import {
        /// Input BibTeX file
        filename: String
    },

    /// Add a tag to a reference
    Tag {
        /// Entry key, full id or short id
        input: String,
        
        /// Tag name
        tag: String
    },
    
    /// Attach content to a reference
    Attach {
        /// Entry key, full id or short id
        input: String,

        /// Content location
        location: String,
    },
    
    /// Open reference content
    Open {
        /// Entry key, full id or short id
        input: String,
    },
}

impl Cli {
    pub fn run(self, bark: &Bark) -> Result<(), Box<dyn std::error::Error>> {
        let conn = bark.conn();

        match self.command {
            Commands::Add => {
                println!("Paste BibTeX, Ctrl+D to finish:\n");

                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;

                let id = service::add_reference(conn, &input)?;
                println!("Saved as {}", id);
            }

            Commands::List { tag } => {
                let refs = service::list_references(conn, tag.as_deref())?;

                let max_key = refs.iter().map(|r| r.key.len()).max().unwrap_or(0);

                for r in refs {
                    let short_id = &r.id[..8];
                    let title = r.title.unwrap_or_else(|| "<no title>".to_string());

                    let tag_str = if r.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", r.tags.join(", "))
                    };

                    println!(
                        "{:8}  {:width$}  {}{}",
                        short_id,
                        r.key,
                        title,
                        tag_str,
                        width = max_key,
                    );
                }
            }

            Commands::Show { input } => {
                let bib = service::get_reference(conn, &input)?;
                println!("{}", bib);

                match service::get_content(conn, &input) {
                    Ok((kind, location)) => {
                        println!("\n---");
                        println!("content: {} ({})", location, kind);
                    }
                    Err(_) => {
                        // No content (stay silent)
                    }
                }
            }

            Commands::Export { tag } => {
                let content = service::export_references(conn, tag.as_deref())?;
                fs::write("references.bib", content)?;
            }

            Commands::Import { filename } => {
                let result = service::import_bibtex(conn, &filename)?;
                println!("Imported: {} | Skipped: {}", result.added, result.skipped);
            }

            Commands::Tag { input, tag } => {
                service::add_tag(conn, &input, &tag)?;
            }

            Commands::Attach{ input, location } => {
                service::add_content(conn, &input, &location)?;
            }

            Commands::Open { input } => {
                let (kind, location) = service::get_content(conn, &input)?;

                match kind.as_str() {
                    "url" | "pdf" | "local" => {
                        std::process::Command::new("xdg-open")
                            .arg(&location)
                            .spawn()?;
                    }
                    "ssh" => {
                        // location: user@host:/path/to/file
                        let mut parts = location.splitn(2, ":");
                        let host = parts.next().ok_or("Invalid ssh location")?;
                        let path = parts.next().ok_or("Invalid ssh location")?;

                        // Copy file and open locally
                        let tmp_path: PathBuf = env::temp_dir().join("bark_tmp.pdf");
                        std::process::Command::new("scp")
                            .arg(format!("{}:{}", host, path))
                            .arg(&tmp_path)
                            .status()?;
                        std::process::Command::new("xdg-open")
                            .arg(&tmp_path)
                            .spawn()?;
                    }
                    _ => {
                        eprintln!("Unknown content kind: {}", kind)
                    }
                }
            }
        }

        Ok(())
    }
}
