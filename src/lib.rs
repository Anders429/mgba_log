//! A logging implementation for mGBA.
//!
//! Provides a logging implementation for the [`log`] crate for logging when compiling for the Game
//! Boy Advance and running within the [mGBA](https://mgba.io/) emulator.
//!
//! mGBA supports logging at the following log levels using the associated logging macros. Every
//! level provided by the `log` crate is supported except for `Trace`, as mGBA has no analog for
//! the `Trace` log level, and this crate provides a macro for logging directly to `Fatal`.
//!
//! | Level | Macro                | Notes                                                                             |
//! | ----- | -------------------- | --------------------------------------------------------------------------------- |
//! | Debug | [`log::debug!`]      |                                                                                   |
//! | Info  | [`log::info!`]       |                                                                                   |
//! | Warn  | [`log::warn!`]       |                                                                                   |
//! | Error | [`log::error!`]      |                                                                                   |
//! | Fatal | [`mgba_log::fatal!`] | Not a standard [`log`] level. Only usable when using this logging implementation. |
//!
//! # Compatibility
//! This logger uses memory mapped IO registers specific to the Game Boy Advance. It is therefore
//! only safe to use this library when building to run on the Game Boy Advance or a Game Boy
//! Advance emulator.
//!
//! If this logger is attempted to be initialized when not running on mGBA, it will fail to
//! initialize with an [`Error`] identifying the failure.
//!
//! [`mgba_log::fatal!`]: fatal!

#![no_std]
#![warn(clippy::pedantic, missing_docs)]
#![allow(
    // Clippy erroneously believes "mGBA" is an item that requires backticks.
    clippy::doc_markdown,
)]

use core::{
    convert::Into,
    fmt,
    fmt::{write, Display, Write},
    sync::{atomic, atomic::compiler_fence},
};
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};

/// Buffer for log messages to be written to.
const MGBA_LOG_BUFFER: *mut u8 = 0x04FF_F600 as *mut u8;
/// Send register.
///
/// Writing a level to this address drains the log buffer, logging it at the given log level.
const MGBA_LOG_SEND: *mut Level = 0x04FF_F700 as *mut Level;
/// Register for enabling logging.
///
/// Writing a value of `0xC0DE` to this address will initialize logging. If logging was initialized
/// properly in mGBA, reading this address will return the value `0x1DEA`.
const MGBA_LOG_ENABLE: *mut u16 = 0x04FF_F780 as *mut u16;
/// Interrupt Master Enable.
///
/// This register allows enabling and disabling interrupts.
const IME: *mut bool = 0x0400_0208 as *mut bool;

/// A log level within mGBA.
///
/// The enum values correspond to their values within mGBA's logging system. Therefore, these
/// values can simply be written directly to `MGBA_LOG_SEND`.
#[derive(Clone, Copy, Debug)]
enum Level {
    /// Fatal causes mGBA to halt execution.
    Fatal = 0x100,
    Error = 0x101,
    Warning = 0x102,
    Info = 0x103,
    Debug = 0x104,
}

/// Attempt to convert a generic `log::Level` to an mGBA-compatible level.
///
/// This will succeed for every level except `Trace`. mGBA's log system does not have a level
/// analogous to `Trace`.
impl TryFrom<log::Level> for Level {
    type Error = ();

    /// Can only fail when `level == log::Level::Trace`.
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

/// Writes bytes directly to mGBA's log buffer for a given level.
///
/// This writer automatically handles flushing the buffer when it is at capacity (256 bytes).
#[derive(Debug)]
struct Writer {
    /// The mGBA log level of the bytes written by this writer.
    ///
    /// A new writer should be created for each new log level.
    level: Level,

    /// The current position within the log buffer.
    index: u8,
}

impl Writer {
    /// Creates a new writer for the given mGBA log level.
    fn new(level: Level) -> Self {
        Self { level, index: 0 }
    }

    fn write_byte(&mut self, byte: u8) {
        // Write the new byte.
        // SAFETY: This is guaranteed to be valid and in-bounds.
        unsafe {
            MGBA_LOG_BUFFER
                .add(self.index as usize)
                .write_volatile(byte);
        }

        let (index, overflowed) = self.index.overflowing_add(1);
        self.index = index;
        if overflowed {
            // SAFETY: This is guaranteed to be a write to a valid address.
            unsafe {
                MGBA_LOG_SEND.write_volatile(self.level);
            }
        }
    }

    fn send(&mut self) {
        // Write a null byte, indicating that this is the end of the message.
        self.write_byte(b'\x00');
        if self.index != 0 {
            // SAFETY: This is guaranteed to be a write to a valid address.
            unsafe {
                MGBA_LOG_SEND.write_volatile(self.level);
            }
            self.index = 0;
        }
    }
}

impl Write for Writer {
    /// Write the given string to the log buffer.
    ///
    /// The buffer is flushed automatically when it becomes full.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &byte in s.as_bytes() {
            match byte {
                b'\n' => {
                    // For readability purposes, just start a new log line.
                    self.send();
                }
                b'\x00' => {
                    // mGBA interprets null as the end of a line, so we replace null characters
                    // with substitute characters when they are intentionally logged.
                    self.write_byte(b'\x1a');
                }
                _ => {
                    self.write_byte(byte);
                }
            }
        }
        Ok(())
    }
}

impl Drop for Writer {
    /// Flushes the buffer, ensuring that the remaining bytes are sent.
    fn drop(&mut self) {
        self.send();
    }
}

/// Implements the logging interface for mGBA logging.
///
/// This struct implements `log::Log`, allowing it to be used as a logger with the `log` crate.
/// Logging can be done using the standard log interface.
///
/// Note that this logger does not support `log::trace!`, since there are no trace logs available
/// on mGBA.
#[derive(Debug)]
struct Logger;

impl Log for Logger {
    /// Logging is enabled for all log messages besides those whose level is `Trace`.
    ///
    /// This is because there is no analog for the `Trace` log level within mGBA.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    /// Directly logs the `record` to mGBA's memory mapped IO registers for logging.
    ///
    /// Buffer flushing is handled automatically during logging.
    fn log(&self, record: &Record) {
        if let Ok(level) = Level::try_from(record.level()) {
            // Disable interrupts, storing the previous value.
            //
            // This prevents synchronization issues when messages are logged in interrupt handling.
            // Interrupts triggered during this time will be handled when interrupts are reenabled.
            let previous_ime = unsafe { IME.read_volatile() };
            unsafe { IME.write_volatile(false) };

            // Write log record.
            //
            // Note that the writer is dropped after this, causing the buffer to be flushed.
            write(&mut &mut Writer::new(level), *record.args())
                .unwrap_or_else(|error| panic!("write to mGBA log buffer failed: {}", error));

            // Restore previous interrupt enable value.
            unsafe {
                IME.write_volatile(previous_ime);
            }
        }
    }

    /// This is a no-op. Flushing of buffers is already done in `log()`.
    fn flush(&self) {}
}

/// Logs a message at the fatal level.
///
/// `Fatal` is a level specific to mGBA, and is not present within the standard `log` ecosystem.
/// This macro allows logging at this level specifically.
///
/// If [`init()`] has not been successfully run, this will have no effect.
///
/// Note that successfully logging at the `Fatal` level in mGBA will permanently halt execution and
/// display the logged message to the user. As such, it is not possible to log more than 256 bytes,
/// as the execution will be halted as soon as the first 256 bytes in the buffer are flushed.
#[macro_export]
macro_rules! fatal {
    ($($arg:tt)+) => ($crate::__fatal(format_args!($($arg)+)));
}

/// Logs a message at the fatal level.
///
/// This is an implementation detail of the [`fatal!`] macro. It is not considered part of the
/// public API and should not be used directly by external code.
#[doc(hidden)]
pub fn __fatal(args: fmt::Arguments) {
    // Ensure mGBA is listening.
    // SAFETY: This is guaranteed to be a valid read.
    if unsafe { MGBA_LOG_ENABLE.read_volatile() } == 0x1DEA {
        // Disable interrupts.
        //
        // This prevents synchronization issues when messages are logged in interrupt handling.
        unsafe { IME.write_volatile(false) };

        // Fatal logging is often used in panic handlers, so panicking on write failures would lead
        // to recursive panicking. Instead, this fails silently.
        #[allow(unused_must_use)]
        {
            write(&mut Writer::new(Level::Fatal), args);
        }

        // `IME` is not reenabled, because writing with `Level::Fatal` will always cause mGBA to
        // halt execution.
    }
}

/// An error occurring during initialization.
#[derive(Debug)]
pub enum Error {
    /// Enabling of logging was not acknowledged by MGBA.
    ///
    /// This likely indicates that the program is not being run in mGBA at all. In many cases, this
    /// may be considered a recoverable error. However, if this error is returned by [`init()`],
    /// then the logger was never actually set, meaning a different logger could potentially be set
    /// instead.
    NotAcknowledgedByMgba,

    /// An error returned by `log::set_logger()`.
    ///
    /// This most often indicates that another logger has already been set by the program.
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
            Self::SetLoggerError(error) => write!(fmt, "`log::set_logger()` error: {error}"),
        }
    }
}

/// A static logger instance.
///
/// When initializing with [`log::set_logger()`], a static reference to a logger must be provided.
/// This static logger can be used as the static reference.
static LOGGER: Logger = Logger;

/// Initialize mGBA logging.
///
/// This function takes control of mGBA's [memory mapped debug IO registers](
/// https://github.com/mgba-emu/mgba/blob/17a549baf2c8100f2c7e7c244996d9ac85d23198/opt/libgba/mgba.c#L31-L33).
/// Any data previously stored in the debug buffer will be completely overwritten by calls to the
/// [`log`] macros.
///
/// # Errors
/// This function returns `Ok(())` if the logger was enabled. If the logger was not enabled for any
/// reason, it instead returns an [`Error`]. See the documentation for [`Error`] for what errors
/// can occur.
pub fn init() -> Result<(), Error> {
    // SAFETY: This is guaranteed to be a valid write.
    unsafe {
        MGBA_LOG_ENABLE.write(0xC0DE);
    }
    // SAFETY: This is guaranteed to be a valid read.
    if unsafe { MGBA_LOG_ENABLE.read_volatile() } != 0x1DEA {
        return Err(Error::NotAcknowledgedByMgba);
    }

    // Disable interrupts, storing the previous value.
    //
    // This prevents an interrupt handler from attempting to set a different logger while
    // `log::set_logger()` is running.
    //
    // Compiler fences are used to prevent these function calls from being reordered during
    // compilation.
    let previous_ime = unsafe { IME.read_volatile() };
    // SAFETY: This is guaranteed to be a valid write.
    unsafe { IME.write_volatile(false) };
    compiler_fence(atomic::Ordering::Acquire);

    // SAFETY: Interrupts are disabled, therefore this call is safe.
    let result = unsafe { log::set_logger_racy(&LOGGER) }
        // The `TRACE` log level is not used by mGBA.
        // SAFETY: Interrupts are disabled, therefore this call is safe.
        .map(|()| unsafe { log::set_max_level_racy(LevelFilter::Debug) })
        .map_err(Into::into);

    compiler_fence(atomic::Ordering::Release);
    // Restore previous interrupt enable value.
    // SAFETY: This is guaranteed to be a valid write.
    unsafe {
        IME.write_volatile(previous_ime);
    }

    result
}
