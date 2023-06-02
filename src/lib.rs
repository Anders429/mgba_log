#![no_std]

use core::{
    fmt,
    fmt::{write, Display, Write},
};
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use voladdress::{Safe, VolAddress, VolBlock};

const MGBA_LOG_BUFFER: VolBlock<u8, Safe, Safe, 256> = unsafe { VolBlock::new(0x04FF_F600) };
const MGBA_LOG_SEND: VolAddress<Level, Safe, Safe> = unsafe { VolAddress::new(0x04FF_F700) };
const MGBA_LOG_ENABLE: VolAddress<u16, Safe, Safe> = unsafe { VolAddress::new(0x04FF_F780) };

#[derive(Clone, Copy)]
pub enum Level {
    /// Fatal causes mGBA to halt execution.
    Fatal = 0x100,
    Error = 0x101,
    Warning = 0x102,
    Info = 0x103,
    Debug = 0x104,
}

impl TryFrom<log::Level> for Level {
    type Error = ();

    /// Can only fail when `level == Level::Trace`.
    fn try_from(level: log::Level) -> Result<Self, <Self as TryFrom<log::Level>>::Error> {
        match level {
            log::Level::Error => Ok(Self::Error),
            log::Level::Warn => Ok(Self::Warning),
            log::Level::Info => Ok(Self::Info),
            log::Level::Debug => Ok(Self::Debug),
            // There is no analog for trace in mGBA's log system.
            log::Level::Trace => Err(()),
        }
    }
}

pub struct Writer {
    level: Level,
    index: u8,
}

impl Writer {
    pub fn new(level: Level) -> Self {
        Self { level, index: 0 }
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut bytes = s.as_bytes().into_iter();

        while let Some(&byte) = bytes.next() {
            match byte {
                b'\n' => {
                    // For readability purposes, just start a new log line.
                    self.index = 0;
                    MGBA_LOG_SEND.write(self.level);
                    continue;
                }
                b'\x00' => {
                    // mGBA interprets null as the end of a line, so we replace null characters
                    // with substitute characters when they are intentionally logged.

                    // SAFETY: This is guaranteed to be in-bounds.
                    unsafe { MGBA_LOG_BUFFER.get(self.index as usize).unwrap_unchecked() }
                        .write(b'\x1a');
                }
                _ => {
                    // SAFETY: This is guaranteed to be in-bounds.
                    unsafe { MGBA_LOG_BUFFER.get(self.index as usize).unwrap_unchecked() }
                        .write(byte);
                }
            }
            let (index, overflowed) = self.index.overflowing_add(1);
            self.index = index;
            if overflowed {
                MGBA_LOG_SEND.write(self.level);
            }
        }
        Ok(())
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        MGBA_LOG_SEND.write(self.level);
    }
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &Record) {
        if let Ok(level) = Level::try_from(record.level()) {
            let mut writer = Writer::new(level);
            write(&mut writer, *record.args())
                .unwrap_or_else(|error| panic!("write to mGBA log buffer failed: {}", error));
        }
    }

    /// This is a no-op. Flushing of buffers is already done in `log()`.
    fn flush(&self) {}
}

#[derive(Debug)]
pub enum Error {
    NotAcknowledgedByMgba,

    SetLoggerError(SetLoggerError),
}

impl From<SetLoggerError> for Error {
    fn from(error: SetLoggerError) -> Self {
        Self::SetLoggerError(error)
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotAcknowledgedByMgba => fmt.write_str("mGBA did not acknowledge initialization"),
            Self::SetLoggerError(error) => write!(fmt, "`log::set_logger()` error: {}", error),
        }
    }
}

static LOGGER: Logger = Logger;

pub fn init() -> Result<(), Error> {
    MGBA_LOG_ENABLE.write(0xC0DE);
    if MGBA_LOG_ENABLE.read() != 0x1DEA {
        return Err(Error::NotAcknowledgedByMgba);
    }
    log::set_logger(&LOGGER)
        // The `TRACE` log level is not used by mGBA.
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .map_err(|error| error.into())
}
