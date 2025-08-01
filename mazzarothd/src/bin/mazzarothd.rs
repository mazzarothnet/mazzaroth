use utils::log::init_log;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_log();

    Ok(())
}
