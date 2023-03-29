use dialoguer::{theme::ColorfulTheme, Select};
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Display, Clone, Copy, EnumIter)]
pub enum NgSelect {
    Enable,
    Disable,
    Edit,
    Test,
    Reload,
}

pub fn ng_select() -> NgSelect {
    let selections: Vec<_> = NgSelect::iter().collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("")
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();

    selections[selection]
}
