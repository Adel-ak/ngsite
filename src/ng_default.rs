use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::{collections::HashMap, path::Path};
use strum::{Display, EnumIter, IntoEnumIterator};
use tokio::fs::{create_dir_all, File};
use tokio::io::AsyncWriteExt;

#[derive(Eq, Hash, PartialEq, Debug, Display, Clone, Copy, EnumIter)]
enum NgDefaults {
    #[strum(serialize = "default server")]
    DefaultServer,
    #[strum(serialize = "nginx.conf")]
    NginxConf,
    #[strum(serialize = "example.com")]
    ExampleCom,
    #[strum(serialize = "proxy.com")]
    ProxyCom,
    #[strum(serialize = "general.conf")]
    GeneralConf,
    #[strum(serialize = "letsencrypt.conf")]
    LetsencryptConf,
    #[strum(serialize = "proxy.conf")]
    ProxyConf,
    #[strum(serialize = "security.conf")]
    SecurityConf,
}

#[derive(Debug, Clone)]
struct FileMetaData {
    file_path: String,
    folder_path: String,
    default_file: &'static [u8],
}

impl FileMetaData {
    fn new(
        folder_path: impl Into<String>,
        file_name: impl Into<String>,
        default_file: &'static [u8],
    ) -> Self {
        let file_name = file_name.into();
        let folder_path = folder_path.into();
        let file_path = format!("{folder_path}/{file_name}");

        Self {
            file_path,
            folder_path,
            default_file,
        }
    }
}

// static HOST_FILE: &'static [u8] = include_bytes!("./defaults/example.com");

pub async fn ng_default() -> Result<()> {
    let default_files: HashMap<NgDefaults, FileMetaData> = HashMap::from([
        (
            NgDefaults::DefaultServer,
            FileMetaData::new(
                "/etc/nginx/sites-available",
                "default_server",
                include_bytes!("./defaults/default_server"),
            ),
        ),
        (
            NgDefaults::ExampleCom,
            FileMetaData::new(
                "/etc/nginx/sites-available",
                "example.com",
                include_bytes!("./defaults/example.com"),
            ),
        ),
        (
            NgDefaults::ProxyCom,
            FileMetaData::new(
                "/etc/nginx/sites-available",
                "proxy.com",
                include_bytes!("./defaults/proxy.com"),
            ),
        ),
        (
            NgDefaults::GeneralConf,
            FileMetaData::new(
                "/etc/nginx/nginxconfig.io",
                "general.conf",
                include_bytes!("./defaults/general.conf"),
            ),
        ),
        (
            NgDefaults::LetsencryptConf,
            FileMetaData::new(
                "/etc/nginx/nginxconfig.io",
                "letsencrypt.conf",
                include_bytes!("./defaults/letsencrypt.conf"),
            ),
        ),
        (
            NgDefaults::ProxyConf,
            FileMetaData::new(
                "/etc/nginx/nginxconfig.io",
                "proxy.conf",
                include_bytes!("./defaults/proxy.conf"),
            ),
        ),
        (
            NgDefaults::SecurityConf,
            FileMetaData::new(
                "/etc/nginx/nginxconfig.io",
                "security.conf",
                include_bytes!("./defaults/security.conf"),
            ),
        ),
    ]);

    let multi_selections: Vec<_> = NgDefaults::iter().collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick one or more")
        .items(&multi_selections[..])
        .interact()?;

    for selection in selections {
        let selected = multi_selections[selection];
        let default_file = default_files.get(&selected).unwrap();

        log::info!("Creating {}...", default_file.file_path);

        let file_exists = Path::new(&default_file.file_path).exists();

        if !file_exists {
            create_dir_all(&default_file.folder_path).await?;
            let mut file = File::create(&default_file.file_path).await?;
            file.write_all(default_file.default_file).await?;
            log::info!("File created...");
        } else {
            log::warn!("{} File already exists...", default_file.file_path);
        }
    }
    Ok(())
}
