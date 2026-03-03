use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    InvalidTime(String),
    InvalidKey(String),
    InvalidTempo(String),
    InvalidVocals(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidTime(time) => write!(f, "invalid time definition: {time}"),
            Error::InvalidKey(key) => write!(f, "invalid key: {key}"),
            Error::InvalidTempo(key) => write!(f, "invalid tempo: {key}"),
            Error::InvalidVocals(voice) => write!(f, "invalid voice definition: {voice}"),
        }
    }
}

impl StdError for Error {}
