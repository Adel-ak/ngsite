use crate::utils::{reload_nginx, test_nginx};
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Display, PartialEq, Clone, Copy, EnumIter)]
enum YesNo {
    Yes,
    No,
}

pub fn ng_test_reload() -> Result<()> {
    let selections: Vec<_> = YesNo::iter().collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Test and Reload?")
        .default(0)
        .items(&selections[..])
        .interact()?;

    let selected = selections[selection];

    if selected == YesNo::Yes {
        test_nginx()?;
        reload_nginx()?;
    } else {
        log::info!("Skipping test and reload...");
    }

    Ok(())
}
