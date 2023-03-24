use std::collections::HashMap;
use std::fs::remove_file;
use std::io::{self, ErrorKind};
use std::os::unix::fs::symlink;
use std::process::Command;
use walkdir::WalkDir;

pub const AVAILABLE: &str = "sites-available";
pub const ENABLED: &str = "sites-enabled";

pub struct FileData {
    pub file_name: String,
    pub is_symlink: bool,
}

pub fn walk_folder(folder: &str) {
    let mut files: HashMap<String, FileData> = HashMap::new();
    let folder_name = format!("/etc/nginx/{}", folder);
    let iter = WalkDir::new(&folder_name).max_depth(1);
    for entry in iter {
        let entry = entry.unwrap();
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
}

pub fn sym_link(file: String) -> Result<(), io::Error> {
    let available_path = format!("/etc/nginx/{}/{}", AVAILABLE, file);
    let enabled_path = format!("/etc/nginx/{}/{}", ENABLED, file);

    let symlink_res = symlink(available_path, enabled_path);

    if let Err(err) = &symlink_res {
        if err.kind() == ErrorKind::AlreadyExists {
            let enabled_path = format!("/etc/nginx/{}/{}", ENABLED, file);

            remove_file(enabled_path)?;
            return sym_link(file);
        }
    }

    symlink_res
}

pub fn rm_symlink(file: String) -> Result<(), io::Error> {
    let enabled_path = format!("/etc/nginx/{}/{}", ENABLED, file);
    remove_file(enabled_path)
}

pub fn test_nginx() {
    let output = Command::new("nginx")
        .arg("-t")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("Nginx test is successful");
    } else {
        println!("Nginx test is failed");
        panic!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

pub fn reload_nginx() {
    let output = Command::new("systemctl")
        .arg("reload")
        .arg("nginx")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("Nginx reload is successful");
    } else {
        println!(
            "Nginx reload
 failed"
        );
        panic!("{}", String::from_utf8_lossy(&output.stderr));
    }
}
