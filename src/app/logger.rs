use log::{LevelFilter, Record, SetLoggerError};
use std::sync::mpsc::*;

pub struct Log {
    pub level: log::Level,
    pub text: String,
}
pub struct Logger {
    // Feeling more like a gopher than a rustacean
    channel: SyncSender<Log>,
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.channel
                .send(Log {
                    level: record.level(),
                    text: record.args().to_string(),
                })
                .expect("SyncChannel failed in single threaded application");
        }
    }

    fn flush(&self) {}
}

pub fn init(level: LevelFilter) -> Result<Receiver<Log>, SetLoggerError> {
    // This program doesn't actually make use of multiple threads so this bound form log::Log bounding Sync
    // If you somehow manage to send over 1000 logs in a single frame there are bigger problems at hand than
    // my dubious choice of logging lib for my single threaded application
    let (tx, rx) = sync_channel(1000);
    log::set_boxed_logger(Box::new(Logger { channel: tx }))?;
    log::set_max_level(level);
    Ok(rx)
}
