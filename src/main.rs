use std::env;
use std::io::prelude::*;
use std::io::{BufReader, Write};
use std::net::TcpStream;

use crate::cli_args::InputArgs;

pub mod cli_args;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args.clone());
    let input_args = InputArgs::try_from(args).expect("invalid args");
    dbg!(&input_args);

    let mut stream = TcpStream::connect((input_args.server_name.clone(), 143)).unwrap();
    let tag = "TestTag";
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    dbg!(&line);

    let _ = stream.write(
        format!(
            "{} LOGIN {} {}\r\n",
            &tag, &input_args.username, &input_args.password
        )
        .as_bytes(),
    );

    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    dbg!(&line);
}
