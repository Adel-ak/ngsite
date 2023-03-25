mod ng_disable_site;
mod ng_enable_site;
mod ng_select;
mod utils;

use anyhow::Result;
use ng_disable_site::ng_disable_site;
use ng_enable_site::ng_enable_site;
use ng_select::{ng_select, NgSelect};
use utils::init_env;

fn main() -> Result<()> {
    init_env();

    let selection = ng_select();
    match selection {
        NgSelect::Enable => ng_enable_site()?,
        NgSelect::Disable => ng_disable_site()?,
    }

    Ok(())
}
