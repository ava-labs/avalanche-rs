pub fn setup_logger(log_level: String) {
    let lvl = match log_level.to_lowercase().as_str() {
        "off" => log::LevelFilter::Off,
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => {
            println!("defaulting {} to 'info' level", log_level);
            log::LevelFilter::Info
        }
    };

    let mut builder = env_logger::Builder::new();
    builder.filter(None, lvl);

    if let Ok(env) = std::env::var("RUST_LOG") {
        builder.parse_filters(&env);
    }

    let _r = builder.try_init();
}
