use anyhow::Context;
use anyhow::{anyhow, Result};
use env_logger::fmt::Color;
use log::Level;
use std::collections::HashMap;
use std::env;
use std::fs::remove_file;
use std::io::ErrorKind;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::process::Command;
use walkdir::WalkDir;

pub const AVAILABLE: &str = "sites-available";
pub const ENABLED: &str = "sites-enabled";

pub struct FileData {
    pub file_name: String,
    pub is_symlink: bool,
}

pub fn walk_folder(folder: &str) -> Result<HashMap<String, FileData>> {
    let mut files: HashMap<String, FileData> = HashMap::new();
    let folder_path = format!("/etc/nginx/{}", folder);
    let iter = WalkDir::new(&folder_path).max_depth(1);
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

pub fn sym_link(file: String) -> Result<()> {
    let available_path = format!("/etc/nginx/{}/{}", AVAILABLE, file);
    let enabled_path = format!("/etc/nginx/{}/{}", ENABLED, file);

    let symlink_res = symlink(available_path, enabled_path);

    if let Err(err) = symlink_res {
        if err.kind() == ErrorKind::AlreadyExists {
            let enabled_path = format!("/etc/nginx/{}/{}", ENABLED, file);

            remove_file(enabled_path)?;
            return sym_link(file);
        }

        log::error!("Failed to symlink");
        return Err(err.into());
    }

    Ok(())
}

pub fn rm_symlink(file: String) -> Result<()> {
    let enabled_path = format!("/etc/nginx/{}/{}", ENABLED, file);
    remove_file(enabled_path)?;
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
    let file_path = format!("/etc/nginx/{}/{}", AVAILABLE, file_name);

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
