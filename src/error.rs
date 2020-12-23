use std::{fmt, io, fs};

#[derive(Debug)]
pub enum EError {
    Io(io::Error),
    Other,
}

impl fmt::Display for EError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(ref err) => f.write_str("Error io"),
            Self::Other => f.write_str("Errror other"),
        }
    }
}

impl std::error::Error for EError {
}

impl From<io::Error> for EError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<crossterm::ErrorKind> for EError {
    fn from(error: crossterm::ErrorKind) -> Self {
        use crossterm::ErrorKind;
        match error {
            ErrorKind::IoError(e) => Self::Io(e),
            _ => Self::Other,
        }
    }
}
