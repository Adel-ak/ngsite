use crate::config::CONFIG;
use anyhow::{anyhow, Context, Result};
use async_compression::tokio::bufread::GzipDecoder;
use async_recursion::async_recursion;
use env_logger::fmt::Color;
use log::Level;
use minus::{page_all, ExitStrategy, LineNumbers, MinusError, Pager};
use serde_json::Value;
use std::collections::HashMap;
use std::env::{set_var, var_os};
use std::fs::remove_file;
use std::io::{ErrorKind, Write};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncReadExt, BufReader};
use walkdir::WalkDir;
use which::which;

#[derive(Debug, Clone)]
pub struct FileData {
    pub file_name: String,
    pub file_path: String,
    pub is_symlink: bool,
}

pub async fn walk_folder(folder: &str) -> Result<HashMap<String, FileData>> {
    let mut files: HashMap<String, FileData> = HashMap::new();
    let dir = Path::new(folder);

    if !dir.exists() {
        create_dir_all(&dir).await?;
    }

    let iter = WalkDir::new(dir).max_depth(1);
    for entry in iter {
        let entry = entry?;
        let file_name: String = entry.file_name().to_string_lossy().into();
        let file_path: String = entry.path().to_string_lossy().into();
        let is_symlink = entry.path_is_symlink();
        let is_dir = entry.into_path().is_dir();

        if is_dir {
            continue;
        }

        let key = file_name.clone();
        let value = FileData {
            file_name,
            file_path,
            is_symlink,
        };

        files.insert(key, value);
    }

    Ok(files)
}

#[async_recursion]
pub async fn sym_link(file: String) -> Result<()> {
    let available_dir = Path::new(&CONFIG.paths.sites_available);
    let enabled_dir = Path::new(&CONFIG.paths.sites_enabled);

    if !enabled_dir.exists() {
        create_dir_all(&enabled_dir).await?;
    }

    if !available_dir.exists() {
        create_dir_all(&available_dir).await?;
    }

    let enabled_path = enabled_dir.join(&file);
    let available_path = available_dir.join(&file);

    if !available_path.exists() {
        error!("Failed to symlink, file not found");

        return Ok(());
    }

    let symlink_res = symlink(&available_path, &enabled_path);

    if let Err(err) = symlink_res {
        if err.kind() == ErrorKind::AlreadyExists {
            remove_file(enabled_path)?;
            return sym_link(file).await;
        }

        error!("Failed to symlink");
        return Err(err.into());
    }

    Ok(())
}

pub async fn rm_symlink(file_name: String) -> Result<()> {
    let enabled_dir = Path::new(&CONFIG.paths.sites_available);
    let file_path = enabled_dir.join(file_name);

    if file_path.exists() {
        remove_file(file_path)?;
    }

    Ok(())
}

pub fn test_nginx() -> Result<()> {
    let nginx_path = get_command_path("nginx")?;
    let output = Command::new(nginx_path).arg("-t").output()?;

    if !output.status.success() {
        error!("Nginx test failed");
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(anyhow!(err));
    }

    info!("Nginx test is successful");
    Ok(())
}

pub fn reload_nginx() -> Result<()> {
    let systemctl_path = get_command_path("systemctl")?;
    let output = Command::new(systemctl_path)
        .arg("reload")
        .arg("nginx")
        .output()?;

    if !output.status.success() {
        error!("Nginx reload  failed");

        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(anyhow!(err));
    }

    info!("Nginx reloaded successfully");
    Ok(())
}

pub fn edit_nginx_site(file_name: String) -> Result<()> {
    let available_dir = Path::new(&CONFIG.paths.sites_available);
    let file_path = available_dir.join(file_name);

    if !file_path.exists() {
        info!("File not found.");

        return Ok(());
    }

    let vi_path = get_command_path("vi")?;
    let output = Command::new(vi_path).arg(file_path).status()?;

    if !output.success() {
        error!("Edit failed");

        let err = output.to_string();
        return Err(anyhow!(err));
    }

    info!("Edit done.");
    Ok(())
}

pub async fn view_nginx_site(file_name: String) -> Result<()> {
    let available_dir = Path::new(&CONFIG.paths.sites_available);
    let file_path = available_dir.join(&file_name);

    if !file_path.exists() {
        info!("File not found.");

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

pub async fn view_log_file(file_name: String) -> Result<()> {
    let logs_dir = Path::new(&CONFIG.paths.logs);
    let file_path = logs_dir.join(&file_name);

    if !file_path.exists() {
        info!("File not found.");

        return Ok(());
    }

    let log = if is_gzip(file_path.to_string_lossy())? {
        read_gzip_log(file_path).await?
    } else {
        read_log(file_path).await?
    };

    cli_pager(log, &file_name).await?;

    // let cat_path = get_command_path(command)?;

    // let output = Command::new(cat_path).arg(file_path).status()?;

    // if !output.success() {
    //     error!("Edit viewing log");

    //     let err = output.to_string();
    //     return Err(anyhow!(err));
    // }

    // Command::new(command)
    //     .arg("-vt")
    //     .arg(&file.into())
    //     .output()?;

    // let mut file = File::open(file_path).await?;
    // let mut buffer = String::new();

    // file.read_to_string(&mut buffer).await?;

    // println!("---------------------- Start of {file_name} ----------------------");
    // println!("{buffer}");
    // println!("---------------------- End of {file_name} ----------------------");
    Ok(())
}

async fn read_gzip_log(file_path: PathBuf) -> Result<String> {
    let mut buffer = String::new();

    let file = File::open(file_path).await?;
    let file = BufReader::new(file);
    let mut file = GzipDecoder::new(file);

    file.read_to_string(&mut buffer).await?;

    Ok(buffer)
}

async fn read_log(file_path: PathBuf) -> Result<String> {
    let mut buffer = String::new();

    let mut file = File::open(file_path).await?;

    file.read_to_string(&mut buffer).await?;

    Ok(buffer)
}

pub fn init_logger() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    let var_key = "NGSITE_LOG";

    if var_os(var_key).is_none() {
        let var_value = ["debug"];
        set_var(var_key, var_value.join(","));
    }

    env_logger::builder()
        .format(|buf, record| {
            let mut style = buf.style();
            let level = record.level();
            let style_color = match level {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                Level::Info => Color::Green,
                Level::Debug => Color::Blue,
                Level::Trace => Color::Magenta,
            };

            style.set_color(style_color);

            writeln!(buf, "[{}] - {}", style.value(level), record.args())
        })
        .init();
}

pub fn get_command_path(path: impl Into<String>) -> Result<PathBuf> {
    let path_string: String = path.into();
    let path_result = which(&path_string).context(format!("{path_string} not found"))?;
    Ok(path_result)
}

pub fn merge_config(
    default_config: &mut Value,
    user_config: Value,
) {
    match (default_config, user_config) {
        (a @ &mut Value::Object(_), Value::Object(b)) => {
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                merge_config(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a, b) => *a = b,
    }
}

pub fn is_root() -> bool {
    users::get_current_uid() == 0
}

pub fn is_gzip(file: impl Into<String>) -> Result<bool> {
    let gzip_path = get_command_path("gzip")?;

    let output = Command::new(gzip_path)
        .arg("-vt")
        .arg(&file.into())
        .output()?;

    let err = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(!err.contains("NOT"))
}

pub async fn cli_pager(
    log: String,
    prompt: &String,
) -> Result<(), MinusError> {
    let pager = Pager::new();

    pager.set_exit_strategy(ExitStrategy::PagerQuit)?;
    pager.set_line_numbers(LineNumbers::AlwaysOn)?;
    pager.set_prompt(prompt)?;
    pager.set_run_no_overflow(true)?;

    let ignore_values_in_log = CONFIG.ignore_values_in_log.iter();
    let ignore_values_in_log_ref = ignore_values_in_log.as_ref();

    'outer: for line in log.lines().rev() {
        for skip_value in ignore_values_in_log_ref {
            if line.contains(skip_value) {
                continue 'outer;
            }
        }

        pager.push_str(line)?;
        pager.push_str("\n")?;
    }

    page_all(pager)?;

    Ok(())
}
