use std::convert::From;
use std::error;
use std::fmt;
use std::io;

#[cfg(feature = "serialport")]
use serialport;

/// Result type used in the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Types of errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// IO error
    Io(io::ErrorKind),

    /// Invalid provided input.
    InvalidInput,

    /// The response from the Sabertooth is invalid.
    Response,

    Unknwown,

    /// Serial error. Its embedded kind is defined by the `serialport` crate.
    #[cfg(feature = "serialport")]
    Serial(serialport::ErrorKind),
}

#[derive(Debug)]
enum SubError {
    None,
    Io(io::Error),

    #[cfg(feature = "serialport")]
    Serial(serialport::Error),
}

/// Error type used in the crate.
#[derive(Debug)]
pub struct Error {
    /// The kind of error this is
    pub kind: ErrorKind,

    /// A description of the error suitable for end-users
    pub description: String,

    source: SubError,
}

impl Error {
    /// Instantiates a new error
    pub fn new<T: Into<String>>(kind: ErrorKind, description: T) -> Self {
        Error {
            kind,
            description: description.into(),
            source: SubError::None,
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.description
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.source {
            #[cfg(feature = "serialport")]
            SubError::Serial(err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(fmt, "{}", &self.description)
    }
}

#[cfg(feature = "serialport")]
impl From<serialport::Error> for Error {
    fn from(err: serialport::Error) -> Error {
        Error {
            kind: ErrorKind::Serial(err.kind),
            description: err.description.clone(),
            source: SubError::Serial(err),
        }
    }
}

#[cfg(feature = "serialport")]
impl From<Error> for serialport::Error {
    fn from(err: Error) -> serialport::Error {
        let kind = match err.kind {
            ErrorKind::Serial(serial_kind) => serial_kind,
            _ => serialport::ErrorKind::Unknown,
        };
        serialport::Error::new(kind, err.description)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error {
            kind: ErrorKind::Io(err.kind()),
            description: err.to_string(),
            source: SubError::Io(err),
        }
    }
}
