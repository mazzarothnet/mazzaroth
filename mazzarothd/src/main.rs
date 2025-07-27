use utils::log::init_log;

pub mod gossip;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_log();

    Ok(())
}
