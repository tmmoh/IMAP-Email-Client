use std::io;

pub type Result<T> = core::result::Result<T, Error>;

pub enum Error {
    TCPConnection(io::Error),
    MissingRead,
    MissingWrite,
    LoginFailed,
    MessageNotFound,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::TCPConnection(err)
    }
}
