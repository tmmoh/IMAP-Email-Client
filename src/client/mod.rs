mod error;

use crate::cli_args::{self, ClientBuilder, Command};

pub use self::error::{Error, Result};

#[derive(Debug)]
pub struct Client {
    pub username: String,
    pub password: String,
    pub folder: String,
    pub message_num: String,
    pub tls: bool,
    pub command: Command,
    pub server_name: String,
}

impl TryFrom<Vec<String>> for Client {
    type Error = cli_args::Error;

    fn try_from(mut args: Vec<String>) -> cli_args::Result<Self> {
        args.remove(0);
        let mut args_iter = args.into_iter();
        let mut args_builder = ClientBuilder::default();

        let mut next_arg = || args_iter.next().ok_or(cli_args::Error::Missing);

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