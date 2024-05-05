use std::env;
use std::io::{prelude::*, BufWriter};
use std::io::{BufReader, Write};
use std::net::TcpStream;

use crate::cli_args::Command;
use crate::client::Client;

pub mod cli_args;
pub mod client;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args.clone());
    let input_args = Client::try_from(args).expect("invalid args");
    dbg!(&input_args);

    let stream = TcpStream::connect((input_args.server_name.clone(), 143)).unwrap();

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = BufWriter::new(stream);

    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    dbg!(&line);

    writer
        .write(
            format!(
                "logintag LOGIN {} {}\r\n",
                &input_args.username, &input_args.password
            )
            .as_bytes(),
        )
        .unwrap();
    writer.flush().unwrap();

    writer
        .write(format!("ftag SELECT {}\r\n", &input_args.folder,).as_bytes())
        .unwrap();
    writer.flush().unwrap();

    match input_args.command {
        Command::Retrieve => {
            writer
                .write(
                    format!("rtag FETCH {} BODY.PEEK[]\r\n", &input_args.message_num,).as_bytes(),
                )
                .unwrap();
        }
        Command::Parse => todo!(),
        Command::Mime => todo!(),
        Command::List => todo!(),
    }


    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    dbg!(&line);

    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    dbg!(&line);
    dbg!();
    /*     let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    println!("here");
    println!("{:?}", lines); */
    for line in reader.lines().map_while(Result::ok) {
        dbg!(line);
    }
}
