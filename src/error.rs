use std::convert::From;
use std::error;
use std::fmt;
use std::io;

/// Result type used in the crate.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// IO error
    Io(io::Error),

    /// Invalid input.
    InvalidInput(String),

    /// The response from the Sabertooth is invalid.
    Response(String),

    /// Other error
    Other,

    /// Serial error. Its embedded kind is defined by the `serialport` crate.
    #[cfg(feature = "serialport")]
    Serial(serialport::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        match self {
            Error::Io(e) => write!(fmt, "IO error: {}", e),
            Error::InvalidInput(msg) => write!(fmt, "Invalid input: {}", msg),
            Error::Response(msg) => write!(fmt, "Invalid response from Sabertooth: {}", msg),
            Error::Other => write!(fmt, "Other saberrs error"),

            #[cfg(feature = "serialport")]
            Error::Serial(e) => write!(fmt, "serialport error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::InvalidInput(_) => None,
            Error::Response(_) => None,
            Error::Other => None,
            Error::Serial(e) => Some(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serialport::Error> for Error {
    fn from(e: serialport::Error) -> Self {
        Self::Serial(e)
    }
}
