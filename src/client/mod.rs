mod error;

use std::io::prelude::*;
use std::{
    io::{BufReader, BufWriter},
    net::TcpStream,
};

pub use self::error::{Error, Result};

pub struct Client {
    pub reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl Client {
    pub fn connect(server_name: impl Into<String>) -> Result<Self> {
        let stream = TcpStream::connect((server_name.into().clone(), 143))?;
        Ok(Self {
            reader: BufReader::new(stream.try_clone()?),
            writer: BufWriter::new(stream),
        })
    }

    pub fn login(
        &mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<()> {
        let to_write = format!("logintag LOGIN {} {}\r\n", username.into(), password.into());
        let written = self.writer.write(to_write.as_bytes())?;

        if written != to_write.as_bytes().len() {
            return Err(Error::MissingWrite);
        };

        self.writer.flush()?;

        Ok(())
    }

    pub fn open_folder(&mut self, folder: Option<impl Into<String>>) -> Result<()> {
        let folder = match folder {
            Some(folder) => folder.into(),
            None => "Inbox".to_string(),
        };

        let to_write = format!("ftag SELECT {}\r\n", folder);
        let written = self.writer.write(to_write.as_bytes())?;

        if written != to_write.as_bytes().len() {
            return Err(Error::MissingWrite);
        };

        self.writer.flush()?;

        Ok(())
    }

    pub fn retrieve(&mut self, message_num: Option<u32>) -> Result<()> {
        let n = match message_num {
            Some(n) => n.to_string(),
            None => "*".to_string(),
        };

        let to_write = format!("rtag FETCH {} BODY.PEEK[]\r\n", n);
        let written = self.writer.write(to_write.as_bytes())?;

        if written != to_write.as_bytes().len() {
            return Err(Error::MissingWrite);
        };

        self.writer.flush()?;

        Ok(())
    }
}
