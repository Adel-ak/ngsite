use anyhow::Context;
use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use env_logger::fmt::Color;
use log::Level;
use std::collections::HashMap;
use std::env;
use std::fs::remove_file;
use std::io::ErrorKind;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncReadExt;
use walkdir::WalkDir;

pub const AVAILABLE: &str = "sites-available";
pub const ENABLED: &str = "sites-enabled";

#[derive(Debug)]
pub struct FileData {
    pub file_name: String,
    pub is_symlink: bool,
}

pub async fn walk_folder(folder: &str) -> Result<HashMap<String, FileData>> {
    let mut files: HashMap<String, FileData> = HashMap::new();
    let dir = Path::new("/etc/nginx").join(folder);

    if !dir.exists() {
        create_dir_all(&dir).await?;
    }

    let iter = WalkDir::new(&dir).max_depth(1);
    for entry in iter {
        let entry = entry?;
        let file_name: String = entry.file_name().to_string_lossy().into();
        let is_symlink = entry.path_is_symlink();
        let is_dir = entry.into_path().is_dir();

        if is_dir {
            continue;
        }

        let key = file_name.clone();
        let value = FileData {
            file_name,
            is_symlink,
        };

        files.insert(key, value);
    }

    Ok(files)
}

#[async_recursion]
pub async fn sym_link(file: String) -> Result<()> {
    let available_dir = Path::new("/etc/nginx").join(AVAILABLE);
    let enabled_dir = Path::new("/etc/nginx").join(ENABLED);

    if !enabled_dir.exists() {
        create_dir_all(&enabled_dir).await?;
    }

    if !available_dir.exists() {
        create_dir_all(&available_dir).await?;
    }

    let enabled_path = enabled_dir.join(&file);
    let available_path = available_dir.join(&file);

    if !available_path.exists() {
        log::error!("Failed to symlink, file not found");

        return Ok(());
    }

    let symlink_res = symlink(&available_path, &enabled_path);

    if let Err(err) = symlink_res {
        if err.kind() == ErrorKind::AlreadyExists {
            remove_file(enabled_path)?;
            return sym_link(file).await;
        }

        log::error!("Failed to symlink");
        return Err(err.into());
    }

    Ok(())
}

pub async fn rm_symlink(file_name: String) -> Result<()> {
    let enabled_dir = Path::new("/etc/nginx").join(ENABLED);
    let file_path = enabled_dir.join(file_name);

    if file_path.exists() {
        remove_file(file_path)?;
    }

    Ok(())
}

pub fn test_nginx() -> Result<()> {
    let output = Command::new("nginx")
        .arg("-t")
        .output()
        .context("Unable to test nginx, is it installed?")?;

    if !output.status.success() {
        log::error!("Nginx test failed");
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(anyhow!(err));
    }

    log::info!("Nginx test is successful");
    Ok(())
}

pub fn reload_nginx() -> Result<()> {
    let output = Command::new("systemctl")
        .arg("reload")
        .arg("nginx")
        .output()
        .context("Unable to reload nginx, is it installed?")?;

    if !output.status.success() {
        log::error!("Nginx reload  failed");

        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(anyhow!(err));
    }

    log::info!("Nginx reloaded successfully");
    Ok(())
}

pub fn edit_nginx_site(file_name: String) -> Result<()> {
    let available_dir = Path::new("/etc/nginx").join(AVAILABLE);
    let file_path = available_dir.join(file_name);

    if !file_path.exists() {
        log::info!("File not found.");

        return Ok(());
    }

    let output = Command::new("vi")
        .arg(file_path)
        .status()
        .context("Unable to edit file")?;

    if !output.success() {
        log::error!("Edit failed");

        let err = output.to_string();
        return Err(anyhow!(err));
    }

    log::info!("Edit done.");
    Ok(())
}

pub async fn view_nginx_site(file_name: String) -> Result<()> {
    let available_dir = Path::new("/etc/nginx").join(AVAILABLE);
    let file_path = available_dir.join(&file_name);

    if !file_path.exists() {
        log::info!("File not found.");

        return Ok(());
    }

    let mut file = File::open(file_path).await?;
    let mut buffer = String::new();

    file.read_to_string(&mut buffer).await?;

    println!("---------------------- Start of {file_name} ----------------------");
    println!("{buffer}");
    println!("---------------------- End of {file_name} ----------------------");
    Ok(())
}

pub fn init_env() {
    env::set_var("RUST_LOG", "Info");

    env_logger::builder()
        .format(|buf, record| {
            let mut style = buf.style();

            let color = match record.level() {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                _ => Color::Rgb(0, 144, 55),
            };

            style.set_color(color);

            writeln!(buf, "{}: {}", style.value(record.level()), record.args())
        })
        .init();
}
