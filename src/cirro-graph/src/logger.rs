use colored::Colorize;

pub fn setup_logger(verbose: bool) -> Result<(), fern::InitError> {
    let level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    let _ = fern::Dispatch::new()
        .format(move |out, message, record| {
            let level_str = match record.level() {
                log::Level::Error => "[E]".red().bold(),
                log::Level::Warn => "[W]".yellow().bold(),
                log::Level::Info => "[*]".green().bold(),
                log::Level::Debug => "[D]".blue().bold(),
                log::Level::Trace => "[T]".normal(),
            };
            out.finish(format_args!("{} {}", level_str, message))
        })
        .level(level)
        .filter(|metadata| {
            let target = metadata.target();
            let level = metadata.level();

            // Suppress only info logs from the neo4rs crate
            // This allows us to see debug logs from neo4rs if verbose is enabled
            if target.starts_with("neo4rs") && level == log::Level::Info {
                false
            } else {
                true // Allow everything else
            }
        })
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}
