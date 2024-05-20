mod error;

use std::fmt::Display;
use std::io::prelude::*;
use std::ops::Not;
use std::{
    io::{BufReader, BufWriter},
    net::TcpStream,
};

pub use self::error::{Error, Result};

pub struct Client {
    pub reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    // TODO: make private
}

#[derive(Default)]
struct Header {
    from: String,
    to: Option<String>,
    date: String,
    subject: Option<String>,
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "From: {}\nTo:{}\nDate: {}\nSubject: {}\n",
            self.from,
            match self.to.as_ref() {
                Some(to) => " ".to_owned() + to,
                None => "".to_owned(),
            },
            self.date,
            self.subject.as_ref().unwrap_or(&"<No subject>".to_string())
        )
    }
}

impl TryFrom<String> for Header {
    type Error = Error;

    fn try_from(value: String) -> std::prelude::v1::Result<Self, Self::Error> {
        let fields = value.trim().split("\r\n");

        let mut header: Self = Default::default();

        for field in fields {
            let (name, data) = dbg!(dbg!(field).split_once(": ").ok_or(Error::MalformedHeader)?);
            let data = data.to_owned();
            match name.to_lowercase().trim() {
                "from" => {
                    header.from = data;
                }
                "to" => {
                    header.to = Some(data);
                }
                "date" => {
                    header.date = data;
                }
                "subject" => {
                    header.subject = Some(data);
                }
                _ => return Err(Error::MalformedHeader),
            };
        }

        Ok(header)
    }
}

impl Client {
    const LOGIN_TAG: &'static str = "logtag";
    const FOLDER_TAG: &'static str = "ftag";
    const RETRIEVE_TAG: &'static str = "rtag";
    const PARSE_TAG: &'static str = "ptag";
    const LIST_TAG: &'static str = "ltag";
    const MIME_TAG: &'static str = "mtag";
    const MIME_HEADER_VERIFY_TAG: &'static str = "mhvtag";
    const MIME_BODY_VERIFY_TAG: &'static str = "mbvtag";

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
            if read != res.len() || read == 0 {
                return Err(Error::MissingRead);
            }

            // Check untagged lines
            if res.starts_with('*') {
                if res.contains("}\r\n") {
                    // Check for literal
                    let start = res
                        .find('{')
                        .expect("Line should always have number of octets");
                    let end = res
                        .find('}')
                        .expect("Line should always have number of octets");
                    let to_read = res[start + 1..end].parse::<usize>().unwrap();
                    dbg!(to_read);

                    let mut literal = vec![b'\0'; to_read];
                    self.reader.read_exact(&mut literal)?;
                    res.push_str(std::str::from_utf8(dbg!(&literal)).expect("should be valid utf-8"));
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
            &[&into_literal(username), &into_literal(password)],
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

        let responses = self.send_command(Self::FOLDER_TAG, "SELECT", &[&into_literal(folder)])?;
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

        let responses = self.send_command(Self::RETRIEVE_TAG, "FETCH", &[n, "BODY.PEEK[]"])?;
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
            .find('{')
            .expect("Line should always have number of octets");
        let end = message
            .find('}')
            .expect("Line should always have number of octets");
        let to_read = message[start + 1..end].parse::<usize>().unwrap();
        let mes = &message[end + 3..end + 3 + to_read];

        print!("{}", mes);

        Ok(())
    }

    pub fn parse(&mut self, message_num: Option<u32>) -> Result<()> {
        let n = match message_num {
            Some(n) => n.to_string(),
            None => "*".to_string(),
        };
        let n = n.as_str();

        let responses = dbg!(self.send_command(
            Self::PARSE_TAG,
            "FETCH",
            &[n, "BODY.PEEK[HEADER.FIELDS (FROM TO DATE SUBJECT)]"],
        )?);
        let tagged_res = responses
            .last()
            .expect("responses is always at least one long");

        if !tagged_res
            .to_lowercase()
            .starts_with(&[Self::PARSE_TAG, "ok"].join(" "))
        {
            return Err(Error::MessageNotFound);
        }

        dbg!(responses.first().unwrap().trim());

        let header = responses.first().unwrap();
        let header = header.split_once("}\r\n").unwrap().1;

        // Unfold header
        let header = header.replace("\r\n ", " ").replace("\r\n\t", "\t");

        let header = Header::try_from(header)?;

        print!("{}", header);

        Ok(())
    }

    pub fn list(&mut self) -> Result<()> {
        let mut responses = dbg!(self.send_command(
            Self::LIST_TAG,
            "FETCH",
            &["1:*", "BODY.PEEK[HEADER.FIELDS (SUBJECT)]"],
        )?);
        let tagged_res = responses
            .pop()
            .expect("responses is always at least one long");

        if !tagged_res
            .to_lowercase()
            .starts_with(&[Self::LIST_TAG, "ok"].join(" "))
        {
            return Err(Error::MessageNotFound);
        }

        responses
            .iter()
            .map(|res| -> Result<Option<String>> {
                let res = res.split_once("}\r\n").ok_or(Error::MalformedHeader)?.1;
                let res = res.replace("\r\n ", " ").replace("\r\n\t", "\t");
                let subject = res
                    .trim()
                    .split_once(": ")
                    .map(|(_, data)| data.to_string());

                Ok(subject)
            })
            .enumerate()
            .try_for_each(|(k, v)| -> Result<()> {
                println!(
                    "{}: {}",
                    k + 1,
                    v?.unwrap_or("<No subject>".to_string()).trim()
                );
                Ok(())
            })?;

        Ok(())
    }

    fn verify_mime_header(&mut self, n: &str) -> Result<()> {
        let responses = self.send_command(
            Self::MIME_HEADER_VERIFY_TAG,
            "FETCH",
            &[n, "BODY.PEEK[HEADER.FIELDS (MIME-Version Content-type)]"],
        )?;

        let tagged_res = responses
            .last()
            .expect("responses is always at least one long");

        if !tagged_res
            .to_lowercase()
            .starts_with(&[Self::MIME_HEADER_VERIFY_TAG, "ok"].join(" "))
        {
            return Err(Error::MessageNotFound);
        }

        dbg!(responses.first().unwrap().trim());

        let header = responses.first().unwrap();
        let header = header.split_once("}\r\n").unwrap().1;

        // Unfold header
        let header = header.replace("\r\n ", " ").replace("\r\n\t", "\t");

        // Split
        let (mime, content) = header.split_once("\r\n").ok_or(Error::MalformedHeader)?;

        if mime.contains("1.0").not() || content.contains("multipart/alternative; boundary=").not()
        {
            dbg!((mime, content));
            return Err(Error::MimeHeaderMatchFail);
        }

        Ok(())
    }

    fn find_first_plain(&mut self, n: &str) -> Result<usize> {
        let responses =
            self.send_command(Self::MIME_BODY_VERIFY_TAG, "FETCH", &[n, "BODYSTRUCTURE"])?;

        let tagged_res = responses
            .last()
            .expect("responses is always at least one long");

        if !tagged_res
            .to_lowercase()
            .starts_with(&[Self::MIME_BODY_VERIFY_TAG, "ok"].join(" "))
        {
            return Err(Error::MessageNotFound);
        }

        let res = responses.first().expect("at least two responses expected");

        let mut start: [Option<usize>; 3] = Default::default();
        start[0] =
            res.find("(\"text\" \"plain\" (\"charset\" \"UTF-8\") NIL NIL \"quoted-printable\"");
        start[1] = res.find("(\"text\" \"plain\" (\"charset\" \"UTF-8\") NIL NIL \"7bit\"");
        start[2] = res.find("(\"text\" \"plain\" (\"charset\" \"UTF-8\") NIL NIL \"8bit\"");

        let start = start
            .iter()
            .filter_map(|n| *n)
            .min()
            .ok_or(Error::MimeMatchFail)?;
        let to_count = &res[0..=start];
        let body_num = dbg!(to_count.split(")(")).count();

        Ok(body_num)
    }

    pub fn mime(&mut self, message_num: Option<u32>) -> Result<()> {
        let n = match message_num {
            Some(n) => n.to_string(),
            None => "*".to_string(),
        };
        let n = n.as_str();

        self.verify_mime_header(n)?;
        let body_num = self.find_first_plain(n)?;

        let responses = self.send_command(
            Self::MIME_TAG,
            "FETCH",
            &[n, format!("BODY.PEEK[{body_num}]").as_str()],
        )?;

        let tagged_res = responses
            .last()
            .expect("responses is always at least one long");

        if !tagged_res
            .to_lowercase()
            .starts_with(&[Self::MIME_TAG, "ok"].join(" "))
        {
            return Err(Error::MessageNotFound);
        }

        // Get message
        let message = responses.first().expect("at least two responses expected");
        let start = message
            .find('{')
            .expect("Line should always have number of octets");
        let end = message
            .find('}')
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
