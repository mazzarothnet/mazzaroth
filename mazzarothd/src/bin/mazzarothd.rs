#![allow(clippy::unwrap_used)]
use mazzarothd::api::serve;
use utils::log::init_log;

#[tokio::main]
async fn main() {
    init().unwrap();
    tokio::spawn(async {
        if let Err(e) = serve().await {
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
