mod ng_disable_site;
mod ng_enable_site;
mod ng_select;
mod utils;

use ng_select::{ng_select, NgSelect};
use std::env;

fn main() {
    env::set_var("RUST_LOG", "Info");

    pretty_env_logger::init();

    let selection = ng_select();
    match selection {
        NgSelect::Enable => {
            log::warn!("Enable")
        }
        NgSelect::Disable => {
            log::error!("Disable")
        }
    }
}
