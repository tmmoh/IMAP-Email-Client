mod error;

use std::str::FromStr;

pub use self::error::{Error, Result};

#[derive(Debug)]
pub struct Args {
    pub username: String,
    pub password: String,
    pub folder: Option<String>,
    pub message_num: Option<u32>,
    pub tls: bool,
    pub command: Command,
    pub server_name: String,
}

impl TryFrom<Vec<String>> for Args {
    type Error = Error;

    fn try_from(mut args: Vec<String>) -> Result<Self> {
        args.remove(0);
        let mut args_iter = args.into_iter();
        let mut args_builder = ArgsBuilder::default();

        let mut next_arg = || args_iter.next().ok_or(Error::Missing);

        args_builder = loop {
            let arg = next_arg()?;
            args_builder = match arg.as_str() {
                "-u" => args_builder.username(next_arg()?)?,
                "-p" => args_builder.password(next_arg()?)?,
                "-f" => args_builder.folder(next_arg()?)?,
                "-n" => args_builder.message_num(next_arg()?.trim().parse()?)?,
                "-t" => args_builder.tls(true)?,
                _ => break args_builder.command(arg)?.server_name(next_arg()?)?,
            };
        };

        args_builder.build()
    }
}

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
pub struct ArgsBuilder {
    username: Option<String>,
    password: Option<String>,
    folder: Option<String>,
    message_num: Option<u32>,
    tls: Option<bool>,
    command: Option<Command>,
    server_name: Option<String>,
}

impl ArgsBuilder {
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

    pub fn build(self) -> Result<Args> {
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

        Ok(Args {
            username,
            password,
            folder: self.folder,
            message_num: self.message_num,
            tls: self.tls.unwrap_or_default(),
            command,
            server_name,
        })
    }
}
