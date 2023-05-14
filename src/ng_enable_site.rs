use crate::utils::{reload_nginx, sym_link, test_nginx, walk_folder, FileData, AVAILABLE, ENABLED};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, MultiSelect};

async fn get_site_names() -> Result<Vec<FileData>> {
    let mut list: Vec<FileData> = vec![];
    let available = walk_folder(AVAILABLE).await?;
    let enabled = walk_folder(ENABLED).await?;

    for (key, file) in available {
        if let Some(enabled_file) = enabled.get(&key) {
            if !enabled_file.is_symlink {
                list.push(file)
            }
        } else {
            list.push(file)
        }
    }

    list.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    Ok(list)
}

pub async fn ng_enable_site() -> Result<()> {
    let list: Vec<FileData> = get_site_names().await?;
    if !list.is_empty() {
        let multi_selections: Vec<String> = list.into_iter().map(|x| x.file_name).collect();

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick site(s)")
            .items(&multi_selections[..])
            .interact()?;

        if !selections.is_empty() {
            for selection in selections {
                let file_to_link = multi_selections[selection].clone();
                sym_link(file_to_link).await?;
            }

            test_nginx()?;
            reload_nginx()?;
        }
    } else {
        log::info!("All sites are enabled...");
    }

    Ok(())
}
