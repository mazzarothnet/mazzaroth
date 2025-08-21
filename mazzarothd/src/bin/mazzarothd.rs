#![allow(clippy::unwrap_used)]
use mazzarothd::{api::serve, state::mz_state::get_mz_state};
use utils::log::init_log;

#[tokio::main]
async fn main() {
    init().unwrap();
    let mz_state = get_mz_state("mazzaroth_data").unwrap();
    let mz_state_clone = mz_state.clone();
    tokio::spawn(async move {
        if let Err(e) = serve(mz_state_clone, mz_state.config.http_port).await {
            eprintln!("Failed to serve API: {}", e);
        }
    });

    tokio::signal::ctrl_c().await.unwrap();
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
