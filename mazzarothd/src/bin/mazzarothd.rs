#![allow(clippy::unwrap_used)]
use mazzarothd::{
    api::serve,
    network::{gossip::spawn_gossip_thread, sync_block::sync_block},
    state::{mvm::spawn_mvm_thread, mz_state::get_mz_state},
};
use utils::log::init_log;

#[tokio::main]
async fn main() {
    init().unwrap();
    let mz_state = get_mz_state("mazzaroth_data").unwrap();
    let mz_state_clone = mz_state.clone();
    tokio::spawn(async move {
        let port = mz_state_clone.config.http_port;
        if let Err(e) = serve(mz_state_clone, port).await {
            eprintln!("Failed to serve API: {}", e);
        }
    });
    let new_block_sender = spawn_gossip_thread(mz_state.clone()).await;
    if let Some(host) = mz_state.config.block_sync_host.as_ref() {
        sync_block(&mz_state, host).await.unwrap();
    }
    spawn_mvm_thread(mz_state.clone());

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
