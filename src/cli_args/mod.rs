mod error;

use std::str::FromStr;
use crate::client::Client;

pub use self::error::{Error, Result};


#[derive(Debug)]
pub enum Command {
    Retrieve,
    Parse,
    Mime,
    List,
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "retrieve" => Ok(Self::Retrieve),
            "parse" => Ok(Self::Parse),
            "mime" => Ok(Self::Mime),
            "list" => Ok(Self::List),
            _ => Err(Error::Invalid),
        }
    }
}


#[derive(Default)]
pub struct ClientBuilder {
    username: Option<String>,
    password: Option<String>,
    folder: Option<String>,
    message_num: Option<u32>,
    tls: Option<bool>,
    command: Option<Command>,
    server_name: Option<String>,
}

impl ClientBuilder {
    pub fn username(mut self, username: impl Into<String>) -> Result<Self> {
        if self.username.is_some() {
            Err(Error::Duplicate)
        } else {
            let _ = self.username.insert(username.into());
            Ok(self)
        }
    }

    pub fn password(mut self, password: impl Into<String>) -> Result<Self> {
        if self.password.is_some() {
            Err(Error::Duplicate)
        } else {
            let _ = self.password.insert(password.into());
            Ok(self)
        }
    }

    pub fn folder(mut self, folder: impl Into<String>) -> Result<Self> {
        if self.folder.is_some() {
            Err(Error::Duplicate)
        } else {
            let _ = self.folder.insert(folder.into());
            Ok(self)
        }
    }

    pub fn message_num(mut self, n: u32) -> Result<Self> {
        if self.message_num.is_some() {
            Err(Error::Duplicate)
        } else {
            let _ = self.message_num.insert(n);
            Ok(self)
        }
    }

    pub fn tls(mut self, tls: bool) -> Result<Self> {
        if self.tls.is_some() {
            Err(Error::Duplicate)
        } else {
            let _ = self.tls.insert(tls);
            Ok(self)
        }
    }

    pub fn command(mut self, command_str: impl Into<String>) -> Result<Self> {
        let command = Command::from_str(&command_str.into())?;

        if self.command.is_some() {
            Err(Error::Duplicate)
        } else {
            let _ = self.command.insert(command);
            Ok(self)
        }
    }

    pub fn server_name(mut self, server_name: impl Into<String>) -> Result<Self> {
        if self.server_name.is_some() {
            Err(Error::Duplicate)
        } else {
            let _ = self.server_name.insert(server_name.into());
            Ok(self)
        }
    }

    pub fn build(self) -> Result<Client> {
        let Some(username) = self.username else {
            return Err(Error::Missing);
        };
        let Some(password) = self.password else {
            return Err(Error::Missing);
        };
        let Some(command) = self.command else {
            return Err(Error::Missing);
        };
        let Some(server_name) = self.server_name else {
            return Err(Error::Missing);
        };

        Ok(Client {
            username,
            password,
            folder: self.folder.unwrap_or("Inbox".to_string()),
            message_num: self.message_num.map_or("*".to_string(), |n| n.to_string()),
            tls: self.tls.unwrap_or_default(),
            command,
            server_name,
        })
    }
}
