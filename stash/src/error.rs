#[derive(Debug)]
pub enum Error {
    ConnectionError(iroh::endpoint::ConnectionError),
    ConnectError(iroh::endpoint::ConnectError),
    CloseError(iroh::endpoint::ClosedStream),
    ReadError(iroh::endpoint::ReadError),
    WriteError(iroh::endpoint::WriteError),
    JsonError(serde_json::Error),
    IoError(std::io::Error),
    DbError(sqlx::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionError(e) => write!(f, "ConnectionError: {:?}", e),
            Self::ConnectError(e) => write!(f, "ConnectError: {:?}", e),
            Self::CloseError(e) => write!(f, "CloseError: {:?}", e),
            Self::WriteError(e) => write!(f, "WriteError: {:?}", e),
            Self::ReadError(e) => write!(f, "ReadError: {:?}", e),
            Self::JsonError(e) => write!(f, "JsonError: {:?}", e),
            Self::IoError(e) => write!(f, "IoError: {:?}", e),
            Self::DbError(e) => write!(f, "DbError: {:?}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<iroh::endpoint::ConnectionError> for Error {
    fn from(value: iroh::endpoint::ConnectionError) -> Self {
        Self::ConnectionError(value)
    }
}

impl From<iroh::endpoint::ConnectError> for Error {
    fn from(value: iroh::endpoint::ConnectError) -> Self {
        Self::ConnectError(value)
    }
}

impl From<iroh::endpoint::ClosedStream> for Error {
    fn from(value: iroh::endpoint::ClosedStream) -> Self {
        Self::CloseError(value)
    }
}

impl From<iroh::endpoint::ReadError> for Error {
    fn from(value: iroh::endpoint::ReadError) -> Self {
        Self::ReadError(value)
    }
}

impl From<iroh::endpoint::WriteError> for Error {
    fn from(value: iroh::endpoint::WriteError) -> Self {
        Self::WriteError(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::DbError(value)
    }
}
