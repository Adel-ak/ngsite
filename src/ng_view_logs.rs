use crate::config::CONFIG;
use crate::utils::{view_log_file, walk_folder, FileData};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};

pub async fn get_site_logs() -> Result<Vec<FileData>> {
    let mut list: Vec<FileData> = vec![];

    let available = walk_folder(&CONFIG.paths.logs).await?;

    for (_, file) in available {
        list.push(file)
    }

    list.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    Ok(list)
}

pub async fn ng_view_logs() -> Result<()> {
    let list: Vec<FileData> = get_site_logs().await?;
    if !list.is_empty() {
        let selections: &Vec<&String> = &list.iter().map(|x| &x.file_name).collect();
        let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick log file")
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        let selected_log = &list[selection];

        view_log_file(selected_log.file_name.clone()).await?;
    } else {
        info!("No sites found to view...");
    }

    Ok(())
}
