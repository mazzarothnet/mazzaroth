#![allow(clippy::unwrap_used)]
use std::sync::atomic::AtomicBool;

use consensus::traits::GENESIS_BLOCK_KEY;
use mazzarothd::{
    api::spawn_api_thread,
    mining::spawn_mining_thread,
    network::{gossip::spawn_gossip_thread, sync_block::sync_block},
    state::{
        mvm::spawn_mvm_thread,
        mz_state::{MzState, get_mz_state},
        tips::force_insert_tips,
    },
};
use utils::log::init_log;

#[tokio::main]
async fn main() {
    init().unwrap();
    let path = "mazzaroth_data";
    //mazzarothd::state::mz_state::clean_block_and_mvm(path).unwrap();
    let mz_state = get_mz_state(path).unwrap();
    spawn_api_thread(mz_state.clone(), mz_state.config.http_port);
    let new_block_sender = spawn_gossip_thread(mz_state.clone()).await;
    if let Some(host) = mz_state.config.block_sync_host.as_ref() {
        match sync_block(&mz_state, host).await {
            Ok(_) => {
                log::info!("Sync block from host {} success", host);
            },
            Err(e) => {
                log::error!("Failed to sync block from host {}: {:?}", host, e);
                log::warn!("Falling back to force inserting genesis block as tip.");
                if let Err(e) = force_insert_tips(vec![GENESIS_BLOCK_KEY], &mz_state) {
                    log::error!("Failed to force insert genesis block as tip: {:?}", e);
                }
            }
        }
    } else {
        if let Err(e) = force_insert_tips(vec![GENESIS_BLOCK_KEY], &mz_state) {
            log::error!("Failed to force insert genesis block as tip: {:?}", e);
        }
    }
    log::info!("spawn_mvm_thread");
    spawn_mvm_thread(mz_state.clone());
    log::info!("spawn_mining_thread");
    spawn_mining_thread(mz_state.clone(), new_block_sender);
    spawn_exit_thread(mz_state.clone());
    tokio::signal::ctrl_c().await.unwrap();
    log::info!("ctrl_c");
    NEED_EXIT.store(true, std::sync::atomic::Ordering::SeqCst);
}

fn init() -> anyhow::Result<()> {
    init_log();
    hook_panic();
    Ok(())
}

fn hook_panic() {
    std::panic::set_hook(Box::new(|info: &std::panic::PanicHookInfo| {
        let message = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => s.as_str(),
                None => "no further details available",
            },
        };

        let location = info
            .location()
            .map_or("unknown location".to_string(), |loc| {
                format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
            });

        eprintln!("panic occurred: '{}' at {}", message, location);
        NEED_EXIT.store(true, std::sync::atomic::Ordering::SeqCst);
    }));
}

static NEED_EXIT: AtomicBool = AtomicBool::new(false);
fn spawn_exit_thread(mz_state: MzState) {
    std::thread::spawn(move || {
        while !NEED_EXIT.load(std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        if let Err(e) = mazzarothd::state::state_dump::dump_blocks(&mz_state, 100) {
            log::error!("dump_blocks error: {:?}", e);
        }
        log::info!("exit");
        std::process::exit(1);
    });
}
