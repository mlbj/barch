use std::process::Command;
use std::path::PathBuf;

use bark_core::{service, Bark};

fn get_sync_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = std::env::var("BARK_SYNC_DIR")?;
    Ok(PathBuf::from(dir))
}

pub fn pull(bark: &Bark) -> Result<(), Box<dyn std::error::Error>> {
    let dir = get_sync_dir()?;

    Command::new("git")
        .arg("-C")
        .arg(&dir)
        .arg("pull")
        .status()?;

    let toml = dir.join("bark.toml");

    service::import_toml(
        bark.conn(),
        toml.to_str().unwrap(),
    )?;

    println!("Sync pull complete");

    Ok(())
}

pub fn push(bark: &Bark) -> Result<(), Box<dyn std::error::Error>> {
    let dir = get_sync_dir()?;

    let toml_content = service::export_all_toml(bark.conn())?;

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
