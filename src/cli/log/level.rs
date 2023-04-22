use clap::ValueEnum;
use log::LevelFilter;

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
    Trace,
    #[default]
    None,
}

impl LogLevel {
    pub fn init_logger(&self) {
        let log = match self {
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::None => LevelFilter::Off,
        };

        env_logger::Builder::new()
            .format_timestamp(None)
            .filter_level(log)
            .init();

        log::info!("setting log level '{}'", log);
    }
}
