//! A publicly exposed library for interoperating with the output of the binary.
//!
//! These types can be used to deserialize the JSON output from the binary. This allows reading the
//! reported log messages.

use serde::{Deserialize, Serialize};

/// The level of a log message.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Level {
    Fatal,
    Error,
    Warning,
    Info,
    Debug,
}

impl TryFrom<u8> for Level {
    type Error = ();

    /// This is the conversion from the actual level value used internally by mGBA. Other values
    /// are possible as well (there are some log levels that are not possible when logging from a
    /// ROM itself); these values simply return an error.
    fn try_from(level: u8) -> Result<Self, ()> {
        match level {
            0x01 => Ok(Self::Fatal),
            0x02 => Ok(Self::Error),
            0x04 => Ok(Self::Warning),
            0x08 => Ok(Self::Info),
            0x10 => Ok(Self::Debug),
            _ => Err(()),
        }
    }
}

/// A single logged message.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Record {
    /// The message's level.
    pub level: Level,
    /// The log message itself.
    pub message: String,
}
