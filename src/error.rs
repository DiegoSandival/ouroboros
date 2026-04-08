use std::fmt;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    ConfigNotFound(PathBuf),
    ConfigParse(toml::de::Error),
    InvalidConfig(&'static str),
    InvalidFileSize { expected: u64, actual: u64 },
    ArithmeticOverflow,
    IndexOutOfBounds { index: u32, max_records: u32 },
    Unauthorized,
    InvalidCell(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::ConfigNotFound(path) => write!(f, "configuration file not found: {}", path.display()),
            Self::ConfigParse(error) => write!(f, "invalid configuration file: {error}"),
            Self::InvalidConfig(message) => write!(f, "invalid configuration: {message}"),
            Self::InvalidFileSize { expected, actual } => {
                write!(f, "invalid data file size: expected {expected} bytes, found {actual}")
            }
            Self::ArithmeticOverflow => write!(f, "arithmetic overflow while computing file offsets"),
            Self::IndexOutOfBounds { index, max_records } => {
                write!(f, "index {index} is out of bounds for {max_records} records")
            }
            Self::Unauthorized => write!(f, "secret does not match the stored record hash"),
            Self::InvalidCell(message) => write!(f, "invalid cell: {message}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::ConfigParse(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Self::ConfigParse(error)
    }
}