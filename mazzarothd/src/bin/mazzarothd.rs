#![allow(clippy::unwrap_used)]
use consensus::{traits::GENESIS_BLOCK_KEY, types::BlockKey};
use mazzarothd::{
    api::spawn_api_thread,
    mining::spawn_mining_thread,
    network::{gossip::spawn_gossip_thread, sync_block::sync_block},
    state::{mvm::spawn_mvm_thread, mz_state::get_mz_state, tips::force_insert_tips},
};
use utils::log::init_log;

#[tokio::main]
async fn main() {
    init().unwrap();
    // let mz_state = get_mz_state("mazzarothd/test_get_block_api").unwrap();
    // let block = mazzarothd::state::block_storage::get_block(
    //     &mz_state.block_storage,
    //     &mazzarothd::state::tips::u32_to_block_key(2),
    // )
    // .unwrap()
    // .unwrap();
    // println!("block: {:?}", block);
    let path = "mazzaroth_data";
    mazzarothd::state::mz_state::clean_block_and_mvm(path).unwrap();
    let mz_state = get_mz_state(path).unwrap();
    spawn_api_thread(mz_state.clone(), mz_state.config.http_port);
    let new_block_sender = spawn_gossip_thread(mz_state.clone()).await;
    if let Some(host) = mz_state.config.block_sync_host.as_ref() {
        sync_block(&mz_state, host).await.unwrap();
    } else {
        force_insert_tips(vec![BlockKey(GENESIS_BLOCK_KEY)], &mz_state).unwrap();
    }
    spawn_mvm_thread(mz_state.clone());
    spawn_mining_thread(mz_state.clone(), new_block_sender);

    tokio::signal::ctrl_c().await.unwrap();
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

        std::process::exit(1);
    }));
}
