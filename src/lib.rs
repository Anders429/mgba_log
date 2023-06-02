#![no_std]

use core::{
    fmt,
    fmt::{write, Write},
};
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use voladdress::{Safe, VolAddress, VolBlock};

const MGBA_LOG_BUFFER: VolBlock<u8, Safe, Safe, 256> =
    unsafe { VolBlock::new(0x04FF_F600) };
const MGBA_LOG_SEND: VolAddress<MgbaLogLevel, Safe, Safe> =
    unsafe { VolAddress::new(0x04FF_F700) };
const MGBA_LOG_ENABLE: VolAddress<u16, Safe, Safe> =
    unsafe { VolAddress::new(0x04FF_F780) };

#[derive(Clone, Copy)]
pub enum MgbaLogLevel {
    /// Fatal causes mGBA to halt execution.
    Fatal = 0x100,
    Error = 0x101,
    Warning = 0x102,
    Info = 0x103,
    Debug = 0x104,
}

impl TryFrom<Level> for MgbaLogLevel {
    type Error = ();

    /// Can only fail when `level == Level::Trace`.
    fn try_from(level: Level) -> Result<Self, <Self as TryFrom<Level>>::Error> {
        match level {
            Level::Error => Ok(Self::Error),
            Level::Warn => Ok(Self::Warning),
            Level::Info => Ok(Self::Info),
            Level::Debug => Ok(Self::Debug),
            // There is no analog for trace in mGBA's log system.
            Level::Trace => Err(()),
        }
    }
}

pub struct MgbaWriter {
    level: MgbaLogLevel,
    index: u8,
}

impl MgbaWriter {
    pub fn new(level: MgbaLogLevel) -> Self {
        Self { level, index: 0 }
    }
}

impl Write for MgbaWriter {
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

impl Drop for MgbaWriter {
    fn drop(&mut self) {
        MGBA_LOG_SEND.write(self.level);
    }
}

struct MgbaLogger;

impl Log for MgbaLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if !matches!(record.level(), Level::Trace) {
            let mut writer = MgbaWriter::new(
                // SAFETY: Just verified above that `record.level()` is not `Trace`.
                unsafe { record.level().try_into().unwrap_unchecked() },
            );
            write(&mut writer, *record.args());
        }
    }

    /// This is a no-op. Flushing of buffers is already done in `log()`.
    fn flush(&self) {}
}

static LOGGER: MgbaLogger = MgbaLogger;

pub fn init() -> Result<(), SetLoggerError> {
    MGBA_LOG_ENABLE.write(0xC0DE);
    log::set_logger(&LOGGER)
        // The `TRACE` log level is not used by mGBA.
        .map(|()| log::set_max_level(LevelFilter::Debug))
}

