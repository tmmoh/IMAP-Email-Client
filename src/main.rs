use std::env;
use std::process::ExitCode;

use crate::cli_args::{Args, Command};
use crate::client::Client;

pub mod cli_args;
pub mod client;

fn main() -> ExitCode {
    let args = Args::try_from(env::args().collect::<Vec<String>>()).expect("invalid args");

    let mut client = match Client::connect(args.server_name) {
        Ok(client) => client,
        Err(_) => return ExitCode::from(1),
    };

    if client.login(&args.username, &args.password).is_err() {
        println!("Login failure");
        return ExitCode::from(3);
    };

    if client.open_folder(args.folder.as_deref()).is_err() {
        return ExitCode::from(3);
    }

    if match args.command {
        Command::Retrieve => client.retrieve(args.message_num),
        Command::Parse => todo!(),
        Command::Mime => todo!(),
        Command::List => todo!(),
    }
    .is_err()
    {
        return ExitCode::from(3);
    }

    ExitCode::from(0)
}
