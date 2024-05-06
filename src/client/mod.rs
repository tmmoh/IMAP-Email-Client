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

    fn send_command(&mut self, tag: &str, command: &str, args: &[&str]) -> Result<Vec<String>> {
        let message = dbg!([&[tag, command], args, &["\r\n"]].concat().join(" "));
        let to_write = message.as_bytes();
        let written = self.writer.write(to_write)?;

        if written != to_write.len() {
            return Err(Error::MissingWrite);
        };

        self.writer.flush()?;

        self.read_until_tag(tag)
    }

    fn read_until_tag(&mut self, tag: &str) -> Result<Vec<String>> {
        let mut responses: Vec<String> = Vec::new();
        let mut res = String::new();
        loop {
            res.clear();
            let read = self.reader.read_line(&mut res)?;
            // Check missing read
            if read != res.len() {
                return Err(Error::MissingRead);
            }

            // Check untagged lines
            if res.starts_with("*") {
                if res.contains("}\r\n") {
                    // Check for literal
                    let start = res
                        .find("{")
                        .expect("Line should always have number of octets");
                    let end = res
                        .find("}")
                        .expect("Line should always have number of octets");
                    let to_read = res[start + 1..end].parse::<usize>().unwrap();
                    dbg!(to_read);

                    let mut literal = vec![b'\0'; to_read];
                    self.reader.read_exact(&mut literal)?;
                    res.push_str(std::str::from_utf8(&literal).expect("should be valid utf-8"));
                }
                responses.push(res.clone());
            } else if res.starts_with(tag) {
                responses.push(res);
                return Ok(responses);
            }
        }
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let responses = self.send_command(
            Self::LOGIN_TAG,
            "LOGIN",
            &[&into_literal(username), &into_literal(password)]
        )?;
        let tagged_res = responses
            .last()
            .expect("responses is always at least one long");

        if !tagged_res
            .to_lowercase()
            .starts_with(&[Self::LOGIN_TAG, "ok"].join(" "))
        {
            return Err(Error::LoginFailed);
        }

        Ok(())
    }

    pub fn open_folder(&mut self, folder: Option<&str>) -> Result<()> {
        let folder = folder.unwrap_or("Inbox");

        let responses =
            self.send_command(Self::FOLDER_TAG, "SELECT", &[&into_literal(folder)])?;
        let tagged_res = responses
            .last()
            .expect("responses is always at least one long");

        if !tagged_res
            .to_lowercase()
            .starts_with(&[Self::FOLDER_TAG, "ok"].join(" "))
        {
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

        let responses =
            self.send_command(Self::RETRIEVE_TAG, "FETCH", &[n, "BODY.PEEK[]"])?;
        let tagged_res = responses
            .last()
            .expect("responses is always at least one long");

        if !tagged_res
            .to_lowercase()
            .starts_with(&[Self::RETRIEVE_TAG, "ok"].join(" "))
        {
            return Err(Error::MessageNotFound);
        }

        // Get message
        let message = responses.first().expect("at least two responses expected");
        let start = message
            .find("{")
            .expect("Line should always have number of octets");
        let end = message
            .find("}")
            .expect("Line should always have number of octets");
        let to_read = message[start + 1..end].parse::<usize>().unwrap();
        let mes = &message[end + 3..end + 3 + to_read];

        print!("{}", mes);

        Ok(())
    }
}

fn into_literal(str: &str) -> String {
    dbg!(format!("{{{}}}\r\n{}", str.len(), str))
}
