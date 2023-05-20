/// Msp error uniform error definition.
#[derive(Debug)]
pub enum MspErr {
    /// Unintended errors occur when processing data.
    DataErr(String),
    /// Internal errors sent, including type conversion, string analysis, etc.
    InternalErr(String),
    /// Unimplemented features.
    NoImpl(String),
    /// Handling errors that occur during sockets.
    IoErr(std::io::Error),
}

impl std::fmt::Display for MspErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MspErr::DataErr(str) => write!(f, "{}", str),
            MspErr::InternalErr(str) => write!(f, "{}", str),
            MspErr::NoImpl(str) => write!(f, "{}", str),
            MspErr::IoErr(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for MspErr {}

impl From<std::io::Error> for MspErr {
    fn from(err: std::io::Error) -> Self {
        MspErr::IoErr(err)
    }
}

impl From<std::time::SystemTimeError> for MspErr {
    fn from(err: std::time::SystemTimeError) -> Self {
        MspErr::InternalErr(err.to_string())
    }
}

impl From<std::num::ParseIntError> for MspErr {
    fn from(err: std::num::ParseIntError) -> Self {
        MspErr::InternalErr(err.to_string())
    }
}
