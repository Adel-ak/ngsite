use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect};
use std::path::PathBuf;
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
    file_path: Box<PathBuf>,
    default_file: &'static [u8],
    file_name: String,
}

impl FileMetaData {
    fn new(
        file_path: impl Into<String>,
        file_name: impl Into<String>,
        default_file: &'static [u8],
    ) -> Self {
        let file_name = file_name.into();
        let file_path = file_path.into();
        let file_path = Path::new(&file_path).join(&file_name);

        Self {
            file_name,
            file_path: Box::new(file_path),
            default_file,
        }
    }
}

// static HOST_FILE: &'static [u8] = include_bytes!("./defaults/example.com");

pub async fn ng_default() -> Result<()> {
    let default_files: HashMap<NgDefaults, FileMetaData> = HashMap::from([
        (
            NgDefaults::NginxConf,
            FileMetaData::new(
                "/etc/nginx",
                "nginx.conf",
                include_bytes!("./defaults/nginx.conf"),
            ),
        ),
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
        let default_file: FileMetaData = match selected {
            NgDefaults::ExampleCom | NgDefaults::ProxyCom => {
                let default_file = default_files.get(&selected).unwrap();

                let domain: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Domain name")
                    .default(default_file.file_name.clone())
                    .interact_text()
                    .unwrap();

                let dir = default_file.file_path.parent().unwrap().to_str().unwrap();

                FileMetaData::new(dir, domain, default_file.default_file)
            }
            _ => default_files.get(&selected).unwrap().clone(),
        };

        let file_path = default_file.file_path;
        let file_name = file_path.file_name().unwrap();

        info!("Creating {:?}...", file_name);

        let file_exists = file_path.exists();

        if !file_exists {
            create_dir_all(&file_path.parent().unwrap()).await?;
            let mut file = File::create(*file_path).await?;

            file.write_all(default_file.default_file).await?;
            info!("File created...");
        } else {
            warn!("{:?} File already exists...", file_name);
        }
    }
    Ok(())
}
