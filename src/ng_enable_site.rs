use crate::utils::{reload_nginx, sym_link, test_nginx, walk_folder, FileData, AVAILABLE, ENABLED};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use walkdir::Error;

fn get_enable_list() -> Result<Vec<FileData>, Error> {
    let mut list: Vec<FileData> = vec![];
    let available = walk_folder(AVAILABLE);
    let enabled = walk_folder(ENABLED);

    for (key, file) in available {
        if let Some(enabled_file) = enabled.get(&key) {
            if !enabled_file.is_symlink {
                list.push(file)
            }
        } else {
            list.push(file)
        }
    }

    Ok(list)
}

pub fn ng_enable_site() -> Result<(), Error> {
    let list: Vec<FileData> = get_enable_list()?;
    if !list.is_empty() {
        let multi_selections: Vec<String> = list.into_iter().map(|x| x.file_name).collect();

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick site(s)")
            .items(&multi_selections[..])
            .interact()
            .unwrap();

        println!("{:?}", selections);
        if !selections.is_empty() {
            for selection in selections {
                let file_to_link = multi_selections[selection].clone();

                if let Err(err) = sym_link(file_to_link) {
                    println!("Failed to symlink");
                    panic!("Err: {:#?}", err);
                }
            }

            test_nginx();
            reload_nginx()
        }
    } else {
        println!("All sites are enabled...");
    }

    Ok(())
}
