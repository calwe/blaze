struct KLogger;

use log::{SetLoggerError, Level, LevelFilter};
use yansi::Paint;

use crate::println;

static LOGGER: KLogger = KLogger;

pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(level))
}

impl log::Log for KLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
           return;
        }

        let level = match record.level() {
            Level::Error => Paint::red("ERROR").bold(),
            Level::Warn => Paint::yellow("WARN ").bold(),
            Level::Info => Paint::green("INFO ").bold(),
            Level::Debug => Paint::blue("DEBUG").bold(),
            Level::Trace => Paint::magenta("TRACE").bold(),
        }; 

        println!("{level} {} > {}", Paint::bright_white(record.target()), record.args());
    }

    fn flush(&self) {}
}
