use std::process::Command;
use std::path::PathBuf;

use bark_core::{service, db, Bark};

fn get_sync_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = std::env::var("BARK_SYNC_DIR")?;
    Ok(PathBuf::from(dir))
}

pub fn restore(bark: &Bark) -> Result<(), Box<dyn std::error::Error>> {
    // Delete everything first
    db::purge(bark.conn());

    let dir = get_sync_dir()?;

    Command::new("git")
        .arg("-C")
        .arg(&dir)
        .arg("pull")
        .status()?;

    let toml_filename = dir.join("bark.toml");
    let toml_content = std::fs::read_to_string(&toml_filename)?;

    service::import_toml(
        bark.conn(),
        &toml_content
    )?;

    println!("Sync restore complete");

    Ok(())
}

pub fn push(bark: &Bark) -> Result<(), Box<dyn std::error::Error>> {
    let dir = get_sync_dir()?;

    let toml_content = service::export_toml_by_tag(bark.conn(), None)?;

    std::fs::write(dir.join("bark.toml"), toml_content)?;

    Command::new("git")
        .arg("-C")
        .arg(&dir)
        .args(["add", "bark.toml"])
        .status()?;

    Command::new("git")
        .arg("-C")
        .arg(&dir)
        .args(["commit", "-m", "bark sync"])
        .status()?;

    Command::new("git")
        .arg("-C")
        .arg(&dir)
        .arg("push")
        .status()?;

    println!("Sync push complete");

    Ok(())
}
