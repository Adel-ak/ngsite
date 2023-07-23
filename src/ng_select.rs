use dialoguer::{theme::ColorfulTheme, Select};
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Debug, Display, Clone, Copy, EnumIter)]
pub enum NgSelect {
    #[strum(serialize = "Create Default")]
    NgDefault,
    #[strum(serialize = "Enable Site")]
    Enable,
    #[strum(serialize = "Disable Site")]
    Disable,
    #[strum(serialize = "View Site")]
    ViewSite,
    #[strum(serialize = "View Log")]
    ViewLog,
    #[strum(serialize = "Edit Site")]
    Edit,
    #[strum(serialize = "Test Nginx")]
    Test,
    #[strum(serialize = "Reload Nginx")]
    Reload,
    Exit,
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
