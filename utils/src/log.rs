use log::info;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};

#[allow(clippy::unwrap_used)]
pub fn init_log() {
    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l} {f}:{L} - {m}{n}")))
        .build("logs/mazzaroth.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(file)))
        .logger(
            Logger::builder()
                .appender("file")
                .additive(false)
                .build("app::module", log::LevelFilter::Debug),
        )
        .build(Root::builder().appender("file").build(
            #[cfg(debug_assertions)]
            log::LevelFilter::Debug,
            #[cfg(not(debug_assertions))]
            log::LevelFilter::Info,
        ))
        .unwrap();

    log4rs::init_config(config).unwrap();

    info!("log init success");
}
