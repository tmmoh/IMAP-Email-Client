use std::num::ParseIntError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Missing,
    Duplicate,
    Invalid,
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Self::Invalid
    }
}