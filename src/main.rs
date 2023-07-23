#[macro_use]
extern crate log;

mod config;
mod ng_default;
mod ng_disable_site;
mod ng_edit_site;
mod ng_enable_site;
mod ng_select;
mod ng_test_reload;
mod ng_view_logs;
mod ng_view_site;
mod utils;

use anyhow::Result;
use ng_default::ng_default;
use ng_disable_site::ng_disable_site;
use ng_edit_site::ng_edit_site;
use ng_enable_site::ng_enable_site;
use ng_select::{ng_select, NgSelect};
use ng_view_logs::ng_view_logs;
use ng_view_site::ng_view_site;
use std::process::{self, exit};
use utils::{init_logger, is_root, reload_nginx, test_nginx};

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    if !is_root() {
        error!("Ngsite require sudo access!");
        exit(1);
    }

    loop {
        run_ngsite().await?;
    }
}

async fn run_ngsite() -> Result<()> {
    let selection = ng_select();

    match selection {
        NgSelect::NgDefault => ng_default().await?,
        NgSelect::Enable => ng_enable_site().await?,
        NgSelect::Disable => ng_disable_site().await?,
        NgSelect::Edit => ng_edit_site().await?,
        NgSelect::ViewSite => ng_view_site().await?,
        NgSelect::ViewLog => ng_view_logs().await?,
        NgSelect::Test => test_nginx()?,
        NgSelect::Reload => reload_nginx()?,
        _ => process::exit(0),
    };

    Ok(())
}
