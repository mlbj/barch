use clap::Parser;
use bark_core::{service, Bark};

mod cli;
use cli::{Cli, Commands};

use std::io::{self, Read};
use std::fs;
use directories::ProjectDirs;
use std::path::PathBuf;

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
    let bark = Bark::new(db_path.to_str().unwrap())?;

    cli.run(&bark)?;

    Ok(())
}
