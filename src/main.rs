use std::env;
use std::io::{prelude::*, BufWriter};
use std::io::{BufReader, Write};
use std::net::TcpStream;

use crate::cli_args::{Command, InputArgs};

pub mod cli_args;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args.clone());
    let input_args = InputArgs::try_from(args).expect("invalid args");
    dbg!(&input_args);

    let mut stream = TcpStream::connect((input_args.server_name.clone(), 143)).unwrap();
    let tag = "TestTag";

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = BufWriter::new(stream);

    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    dbg!(&line);

    writer.write(
        format!(
            "{} LOGIN {} {}\r\n",
            &tag, &input_args.username, &input_args.password
        )
        .as_bytes(),
    ).unwrap();

    writer.write(
        format!(
            "ftag SELECT {}\r\n",
            &input_args.folder,
        )
        .as_bytes(),
    ).unwrap();


    writer.flush().unwrap();

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
    for line in reader.lines().flatten() {
        dbg!(line);
    }
}
