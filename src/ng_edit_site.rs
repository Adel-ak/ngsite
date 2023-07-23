use crate::config::CONFIG;
use crate::ng_test_reload::ng_test_reload;
use crate::utils::{edit_nginx_site, walk_folder, FileData};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};

async fn get_site_names() -> Result<Vec<FileData>> {
    let mut list: Vec<FileData> = vec![];

    let available = walk_folder(&CONFIG.paths.sites_available).await?;

    for (_, file) in available {
        list.push(file)
    }

    list.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    Ok(list)
}

pub async fn ng_edit_site() -> Result<()> {
    let list: Vec<FileData> = get_site_names().await?;
    if !list.is_empty() {
        let selections: &Vec<&String> = &list.iter().map(|x| &x.file_name).collect();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick site")
            .default(0)
            .items(&selections[..])
            .interact()?;

        let selected_site = selections[selection].clone();

        edit_nginx_site(selected_site)?;
        ng_test_reload()?;
    } else {
        info!("No sites found to edit...");
    }

    Ok(())
}
