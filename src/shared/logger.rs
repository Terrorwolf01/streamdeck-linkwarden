// Reusable logging setup for OpenAction plugins. Copy this file into any new plugin's
// src/ and call `logger::init()` at the top of main() - no per-plugin changes needed.
//
// OpenDeck captures a plugin's stdout itself and pipes it into a log file it manages, so
// a plain TermLogger is enough there. The official Elgato Stream Deck software does not
// expose a plugin's stdout/stderr anywhere findable, so without a file logger of our own
// there's nothing to inspect at all when something goes wrong.
//
// There's no reliable way to detect which of the two is actually running the plugin at
// runtime (OpenDeck deliberately reports itself as Stream Deck 7.1.0 in its registration
// info, for plugin compatibility), so this just logs to both unconditionally: whichever
// host is actually running gets the destination it needs, and the other destination goes
// unused. Under OpenDeck that means one extra, redundant log file next to the one it
// already writes for you - a low-enough cost to not bother detecting the host at all.

use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, SharedLogger, TermLogger, TerminalMode, WriteLogger};

/// Sets up logging for an OpenAction plugin: stdout (for OpenDeck's own log capture) plus
/// a `<executable-name>.log` file next to the plugin binary (for real Stream Deck, which
/// has no equivalent). Call this once at the very start of `main()`.
pub fn init() {
    let log_path = std::env::current_exe().ok().map(|mut path| {
        path.set_extension("log");
        path
    });

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Stdout,
        ColorChoice::Never,
    )];

    match log_path.as_ref().map(|path| std::fs::OpenOptions::new().create(true).append(true).open(path)) {
        Some(Ok(file)) => loggers.push(WriteLogger::new(LevelFilter::Debug, Config::default(), file)),
        Some(Err(error)) => eprintln!("Failed to open log file at {:?}: {}", log_path.unwrap(), error),
        None => eprintln!("Failed to determine log file path (could not resolve current executable path)"),
    }

    if let Err(error) = CombinedLogger::init(loggers) {
        eprintln!("Logger initialization failed: {}", error);
    }
}