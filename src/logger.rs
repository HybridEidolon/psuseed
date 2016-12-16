use std::io::{self, Write};
use std::fs::File;
use std::path::Path;
use std::sync::Mutex;

use log::{Log, LogRecord, LogLevel, LogMetadata};

pub struct SimpleLogger {
    file: Mutex<File>
}

impl SimpleLogger {
    pub fn new(path: &'static str) -> io::Result<SimpleLogger> {
        let f = File::create(path)?;
        Ok(SimpleLogger {
            file: Mutex::new(f)
        })
    }
}

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Debug
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let mut f = self.file.lock().unwrap();
            writeln!(f, "[{}] {}\r", record.level(), record.args());
        }
    }
}
