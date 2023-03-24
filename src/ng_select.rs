use dialoguer::{theme::ColorfulTheme, Select};
use strum::Display;

#[derive(Debug, Display, Clone, Copy)]
pub enum NgSelect {
    Enable,
    Disable,
}

pub fn ng_select() -> NgSelect {
    let selections = &[NgSelect::Enable, NgSelect::Disable];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("")
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();

    selections[selection]
}
