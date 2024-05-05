use std::env;
use std::process::ExitCode;

use crate::cli_args::{Args, Command};
use crate::client::Client;

pub mod cli_args;
pub mod client;

fn main() -> ExitCode {
    let args = dbg!(Args::try_from(env::args().collect::<Vec<String>>()).expect("invalid args"));

    let mut client = match Client::connect(&args.server_name) {
        Ok(client) => client,
        Err(_) => return ExitCode::from(1),
    };

    if client.login(&args.username, &args.password).is_err() {
        println!("Login failure");
        return ExitCode::from(3);
    };

    if client.open_folder(args.folder.as_deref()).is_err() {
        println!("Folder not found");
        return ExitCode::from(3);
    }

    match args.command {
        Command::Retrieve => {
            if client.retrieve(args.message_num).is_err() {
                println!("Message not found");
                return ExitCode::from(3);
            }
        }
        Command::Parse => todo!(),
        Command::Mime => todo!(),
        Command::List => todo!(),
    };

    ExitCode::from(0)
}
