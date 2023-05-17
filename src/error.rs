#[derive(Debug)]
pub enum MspErr {
    DataErr(String),
    InternalErr(String),
    NoImpl(String),
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
