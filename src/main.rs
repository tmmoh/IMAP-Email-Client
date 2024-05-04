use std::env;

use crate::cli_args::InputArgs;

pub mod cli_args;


fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args.clone());
    dbg!(InputArgs::try_from(args).expect("invalid args"));
}
