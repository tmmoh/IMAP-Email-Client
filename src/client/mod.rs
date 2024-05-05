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
    const LOGIN_TAG: &'static str = "logtag";
    const FOLDER_TAG: &'static str = "ftag";
    const RETRIEVE_TAG: &'static str = "rtag";

    pub fn connect(server_name: &str) -> Result<Self> {
        let stream = TcpStream::connect((server_name, 143))?;
        Ok(Self {
            reader: BufReader::new(stream.try_clone()?),
            writer: BufWriter::new(stream),
        })
    }

    fn send_command(&mut self, tag: &str, command: &str, args: &[&str]) -> Result<()> {
        let message = dbg!([&[tag, command], args, &["\r\n"]].concat().join(" "));
        let to_write = message.as_bytes();
        let written = self.writer.write(to_write)?;

        if written != to_write.len() {
            return Err(Error::MissingWrite);
        };

        self.writer.flush()?;

        Ok(())
    }

    fn read_until_tag(&mut self, tag: &str) -> Result<String> {
        let mut buf = String::new();
        loop {
            buf.clear();
            let read = self.reader.read_line(&mut buf)?;
            if read != buf.len() {
                return Err(Error::MissingRead);
            }

            if buf.starts_with(tag) {
                return Ok(buf);
            }
        }
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<()> {
        self.send_command(Self::LOGIN_TAG, "LOGIN", &[&into_literal(username), &into_literal(password)])?;
        let line = dbg!(self.read_until_tag(Self::LOGIN_TAG)?);

        if !line.to_lowercase().starts_with(&[Self::LOGIN_TAG, "ok"].join(" ")) {
            return Err(Error::LoginFailed);
        }

        Ok(())
    }

    pub fn open_folder(&mut self, folder: Option<&str>) -> Result<()> {
        let folder = folder.unwrap_or("Inbox");

        self.send_command(Self::FOLDER_TAG, "SELECT", &[&into_literal(folder)])?;
        let line = dbg!(self.read_until_tag(Self::FOLDER_TAG)?);

        if !line.to_lowercase().starts_with(&[Self::FOLDER_TAG, "ok"].join(" ")) {
            return Err(Error::LoginFailed);
        }

        Ok(())
    }

    pub fn retrieve(&mut self, message_num: Option<u32>) -> Result<()> {
        let n = match message_num {
            Some(n) => n.to_string(),
            None => "*".to_string(),
        };
        let n = n.as_str();

        self.send_command(Self::RETRIEVE_TAG, "FETCH", &[n, "BODY.PEEK[]"])?;
        let line = dbg!(self.read_until_tag(&["*", n, "FETCH"].join(" "))?);

        let start = line.find("{").expect("Line should always have number of octets");
        let end = line.find("}").expect("Line should always have number of octets");
        let to_read = line[start+1..end].parse::<usize>().unwrap();
        dbg!(to_read);

        let mut buf = vec![b'\0'; to_read];
        self.reader.read_exact(&mut buf)?;
        print!("{}", String::from_utf8(buf).unwrap());

        Ok(())
    }
}

fn into_literal(str: &str) -> String {
    dbg!(format!("{{{}}}\r\n{}", str.len(), str))
}