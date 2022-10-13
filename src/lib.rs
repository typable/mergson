use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: String,
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: "IOError".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self {
            kind: "JSONParseError".to_string(),
            message: err.to_string(),
        }
    }
}
