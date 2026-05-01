use clap::{Parser, Subcommand};

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
        #[arg(short, long)]
        tag: Option<String> 
    },

    /// Show full BibTeX entry
    Show {
        /// Entry key, full id or short id
        input: String },

    /// Export references to reference.bib file
    Export {
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>
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
}
