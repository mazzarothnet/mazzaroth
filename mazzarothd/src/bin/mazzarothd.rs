#![allow(clippy::unwrap_used)]
use mazzarothd::api::serve;
use mazzarothd::state::{app_data::get_block_db_path, block::BlockStorage};
use utils::log::init_log;

fn main() {
    init().unwrap();
    tokio::spawn(serve());

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn init() -> anyhow::Result<()> {
    init_log();
    hook_panic();
    Ok(())
}

fn hook_panic() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("panic: {:?}", info);
        std::process::exit(1);
    }));
}
