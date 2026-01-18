//! This is where the [`Protocol`](Protocol) structs and its dependencies go.
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use std::str::FromStr;

/// The possible errors that could happen when working with [`Protocol`](Protocol).
#[derive(Debug)]
pub enum Error {
    /// An invalid string was passed to [`Protocol::from_str()`].
    InvalidProtocol(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::InvalidProtocol(protocol) => {
                write!(f, "can't parse '{protocol}' to a valid protocol")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &'static str {
        ""
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

/// This contains every supported protocol by Arch Linux mirror status as of the time of writing
/// (05/20/2021).
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    /// The HTTP protocol.
    #[serde(rename = "http")]
    Http,

    /// The HTTPS protocol.
    #[serde(rename = "https")]
    Https,

    /// The rsync protocol.
    #[serde(rename = "rsync")]
    Rsync,
}

impl FromStr for Protocol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            "rsync" => Ok(Self::Rsync),
            other => Err(Error::InvalidProtocol(other.into())),
        }
    }
}
