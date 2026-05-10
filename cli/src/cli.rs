use clap::{Parser, Subcommand};

use std::io::{self, Read};
use std::fs;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use bark_core::{service, Bark};

use crate::sync;

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

    /// Remove a reference 
    Rm {
        /// Entry key, full id or short id
        input: String,
    },

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

    /// Export references
    Export {
        /// Entry key, full id or short id
        #[arg(index = 1)]
        input: Option<String>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        
        /// Export TOML snapshot
        #[arg(long)]
        toml: bool,
    },

    /// Import references
    Import {
        /// Input file
        filename: String,

        #[arg(long)]
        toml: bool,
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

    /// Sync bark library using a git repository
    Sync {
        /// Restore or push actions
        #[command(subcommand)]
        action: SyncAction,
    },
}

#[derive(Subcommand)]
pub enum SyncAction {
    Restore,
    Push,
}

impl Cli {
    pub fn run(self, bark: &Bark) -> Result<(), Box<dyn std::error::Error>> {
        let conn = bark.conn();

        match self.command {
            Commands::Add => {
                let editor = env::var("BARK_TEXT_EDITOR")
                    .unwrap_or_else(|_| "vim".to_string());

                let tmp_path = env::temp_dir().join("bark_add.toml");

                // Default TOML template
                let template = r#"version = 1

[[references]]
id = ""
bibtex = """
@article{key,
  author = {},
  title = {},
  year = {},
}"""
tags = []

[references.content]
kind = ""
location = ""
"#;

                fs::write(&tmp_path, template);

                Command::new(&editor)
                    .arg(&tmp_path)
                    .status()?;

                let input = fs::read_to_string(&tmp_path)?;

                if input.trim().is_empty() {
                    println!("Aborted (empty entry)");
                    return Ok(());
                }
                service::import_toml(conn, &input)?;
                println!("Imported reference");

                fs::remove_file(&tmp_path).ok();
            }

            Commands::Rm { input } => {
                service::remove_reference(conn, &input)?;
            }

            Commands::List { tag } => {
                let referencess = service::list_references_and_data(conn, tag.as_deref())?;

                let max_key = referencess.iter().map(|r| r.key.len()).max().unwrap_or(0);

                for r in referencess {
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

            Commands::Export { input, tag, toml } => {
                if toml {
                    let content = if let Some(input) = input {
                        service::export_toml(conn, &input)?
                    } else {
                        service::export_toml_by_tag(conn, tag.as_deref())?
                    };

                    fs::write("bark.toml", content)?;
                } else {
                    let content = if let Some(input) = input {
                        service::export_bibtex(conn, &input)?
                    } else {
                        service::export_bibtex_by_tag(conn, tag.as_deref())?
                    };

                    fs::write("references.bib", content)?;
                }
            }

            Commands::Import { filename, toml } => {
                let content = std::fs::read_to_string(&filename)?;
                let path = std::path::Path::new(&filename);
                let extension = path.extension().and_then(|s| s.to_str());

                match (toml, extension) {
                    (true, _) | (false, Some("toml")) => {
                        service::import_toml(conn, &content)?;
                        println!("Imported TOML reference(s)");
                    }
                    (false, Some("bib")) => {

                        let result = service::import_bibtex(conn, &content)?;
                        println!("Imported: {} | Skipped: {}", result.added, result.skipped);
                    }
                    _ => {
                        return Err(format!(
                            "Could not infer format from '{}'. Use --toml or provide a .bib/.toml file",
                            filename
                        ).into());
                    }
                }
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
                    "url" | "local" => {
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

            Commands::Sync { action } => {
                match action {
                    SyncAction::Restore => sync::restore(bark)?,
                    SyncAction::Push => sync::push(bark)?,
                }
            }
        }

        Ok(())
    }
}
