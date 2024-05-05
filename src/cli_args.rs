use std::{num::ParseIntError, str::FromStr};

#[derive(Debug)]
pub enum ArgParseError {
    Missing,
    Duplicate,
    Invalid,
}

impl From<ParseIntError> for ArgParseError {
    fn from(_: ParseIntError) -> Self {
        Self::Invalid
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
    type Err = ArgParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "retrieve" => Ok(Self::Retrieve),
            "parse" => Ok(Self::Parse),
            "mime" => Ok(Self::Mime),
            "list" => Ok(Self::List),
            _ => Err(ArgParseError::Invalid),
        }
    }
}

#[derive(Debug)]
pub struct InputArgs {
    pub username: String,
    pub password: String,
    pub folder: String,
    pub message_num: Option<u32>,
    pub tls: bool,
    pub command: Command,
    pub server_name: String,
}

impl TryFrom<Vec<String>> for InputArgs {
    type Error = ArgParseError;

    fn try_from(mut args: Vec<String>) -> Result<Self, Self::Error> {
        args.remove(0);
        let mut args_iter = args.into_iter();
        let mut args_builder = InputArgsBuilder::default();

        let mut next_arg = || args_iter.next().ok_or(ArgParseError::Missing);

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

#[derive(Default)]
struct InputArgsBuilder {
    username: Option<String>,
    password: Option<String>,
    folder: Option<String>,
    message_num: Option<u32>,
    tls: Option<bool>,
    command: Option<Command>,
    server_name: Option<String>,
}

impl InputArgsBuilder {
    pub fn username(mut self, username: impl Into<String>) -> Result<Self, ArgParseError> {
        if self.username.is_some() {
            Err(ArgParseError::Duplicate)
        } else {
            let _ = self.username.insert(username.into());
            Ok(self)
        }
    }

    pub fn password(mut self, password: impl Into<String>) -> Result<Self, ArgParseError> {
        if self.password.is_some() {
            Err(ArgParseError::Duplicate)
        } else {
            let _ = self.password.insert(password.into());
            Ok(self)
        }
    }

    pub fn folder(mut self, folder: impl Into<String>) -> Result<Self, ArgParseError> {
        if self.folder.is_some() {
            Err(ArgParseError::Duplicate)
        } else {
            let _ = self.folder.insert(folder.into());
            Ok(self)
        }
    }

    pub fn message_num(mut self, n: u32) -> Result<Self, ArgParseError> {
        if self.message_num.is_some() {
            Err(ArgParseError::Duplicate)
        } else {
            let _ = self.message_num.insert(n);
            Ok(self)
        }
    }

    pub fn tls(mut self, tls: bool) -> Result<Self, ArgParseError> {
        if self.tls.is_some() {
            Err(ArgParseError::Duplicate)
        } else {
            let _ = self.tls.insert(tls);
            Ok(self)
        }
    }

    pub fn command(mut self, command_str: impl Into<String>) -> Result<Self, ArgParseError> {
        let command = Command::from_str(&command_str.into())?;

        if self.command.is_some() {
            Err(ArgParseError::Duplicate)
        } else {
            let _ = self.command.insert(command);
            Ok(self)
        }
    }

    pub fn server_name(mut self, server_name: impl Into<String>) -> Result<Self, ArgParseError> {
        if self.server_name.is_some() {
            Err(ArgParseError::Duplicate)
        } else {
            let _ = self.server_name.insert(server_name.into());
            Ok(self)
        }
    }

    fn build(self) -> Result<InputArgs, ArgParseError> {
        let Some(username) = self.username else {
            return Err(ArgParseError::Missing);
        };
        let Some(password) = self.password else {
            return Err(ArgParseError::Missing);
        };
        let Some(command) = self.command else {
            return Err(ArgParseError::Missing);
        };
        let Some(server_name) = self.server_name else {
            return Err(ArgParseError::Missing);
        };

        Ok(InputArgs {
            username,
            password,
            folder: self.folder.unwrap_or("Inbox".to_string()),
            message_num: self.message_num,
            tls: self.tls.unwrap_or_default(),
            command,
            server_name,
        })
    }
}
