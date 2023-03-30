mod ng_default;
mod ng_disable_site;
mod ng_edit_site;
mod ng_enable_site;
mod ng_select;
mod ng_test_reload;
mod utils;

use anyhow::Result;
use ng_default::ng_default;
use ng_disable_site::ng_disable_site;
use ng_edit_site::ng_edit_site;
use ng_enable_site::ng_enable_site;

use ng_select::{ng_select, NgSelect};
use std::process;
use utils::{init_env, reload_nginx, test_nginx};

#[tokio::main]
async fn main() -> Result<()> {
    init_env();

    loop {
        run_ngsite().await?;
    }
}

async fn run_ngsite() -> Result<()> {
    let selection = ng_select();

    match selection {
        NgSelect::NgDefault => ng_default().await?,
        NgSelect::Enable => ng_enable_site()?,
        NgSelect::Disable => ng_disable_site()?,
        NgSelect::Edit => ng_edit_site()?,
        NgSelect::Test => test_nginx()?,
        NgSelect::Reload => reload_nginx()?,
        _ => process::exit(0),
    };

    Ok(())
}
