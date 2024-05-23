use std::env;
use std::process::ExitCode;

use crate::cli_args::{Args, Command};
use crate::client::Client;

pub mod cli_args;
pub mod client;

fn main() -> ExitCode {
    let args: Args = match Args::try_from(env::args().collect::<Vec<String>>()) {
        Ok(args) => args,
        Err(_) => {
            eprintln!("Invalid Args");
            return ExitCode::from(1);
        }
    };

    let mut client = match Client::connect(&args.server_name) {
        Ok(client) => client,
        Err(_) => {
            eprintln!("Cannot Connect to Server");
            return ExitCode::from(2);
        }
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
        Command::Parse => {
            if client.parse(args.message_num).is_err() {
                println!("Message not found");
                return ExitCode::from(3);
            }
        }
        Command::Mime => {
            if let Err(err) = client.mime(args.message_num) {
                match err {
                    client::Error::MessageNotFound => {
                        println!("Message not found");
                        return ExitCode::from(3);
                    }
                    client::Error::MalformedHeader => {
                        println!("Header doesn't contain fields, matching failed");
                        return ExitCode::from(4);
                    }
                    client::Error::MimeMatchFail => {
                        println!("Could not match a message");
                        return ExitCode::from(4);
                    }
                    client::Error::MimeHeaderMatchFail => {
                        println!("Could not match header");
                        return ExitCode::from(4);
                    }
                    _ => {
                        println!("Server communiation error");
                        return ExitCode::from(3);
                    }
                }
            }
        }
        Command::List => {
            if client.list().is_err() {
                return ExitCode::from(0);
            }
        }
    };

    ExitCode::from(0)

    // TODO:
    // Handle message continuation request
}
