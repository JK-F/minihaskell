use std::{env, process::exit};

use eval::eval;
use parser::parse;


fn main() {
    env_logger::init();
    match env::args().collect::<Vec<_>>().get(1) {
        Some(path) => {
            println!("Trying to open {}.", path);
            match std::fs::read_to_string(path) {
                Ok(source) => {
                    let p = parse(&source).unwrap();
                    eval(p).unwrap();
                }
                Err(err) => {
                    eprintln!("Ran into error while trying to open the file.");
                    eprintln!("{}", err);
                },
            }
        }
        None => {
            println!("Please provide the path to a file.");
            exit(1);
        },
    };
}
