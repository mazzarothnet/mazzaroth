use mazzarothd::api::serve;
use utils::log::init_log;

fn main() -> anyhow::Result<()> {
    init()?;
    tokio::spawn(serve());
    // tips问题，咋存，开一个新的db专门存储一些奇怪的数据
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
