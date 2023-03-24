use crate::utils::{reload_nginx, rm_symlink, test_nginx, walk_folder, FileData, ENABLED};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use walkdir::Error;

fn get_enabled_list() -> Result<Vec<FileData>, Error> {
    let mut list: Vec<FileData> = vec![];

    let enabled = walk_folder(ENABLED)?;

    for (_, file) in enabled {
        if file.is_symlink {
            list.push(file)
        }
    }

    Ok(list)
}

pub fn ng_disable_site() -> Result<(), Error> {
    let list: Vec<FileData> = get_enabled_list()?;
    if !list.is_empty() {
        let multi_selections: Vec<String> = list.into_iter().map(|x| x.file_name).collect();

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick site(s)")
            .items(&multi_selections[..])
            .interact()
            .unwrap();

        if !selections.is_empty() {
            for selection in selections {
                let file_to_link = multi_selections[selection].clone();

                if let Err(err) = rm_symlink(file_to_link) {
                    println!("Failed to symlink");
                    panic!("Err: {:#?}", err);
                }
            }

            test_nginx();
            reload_nginx()
        }
    } else {
        println!("All sites are disabled...");
    }

    Ok(())
}
