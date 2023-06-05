use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Record {
    pub level: Level,
    pub message: String,
}
