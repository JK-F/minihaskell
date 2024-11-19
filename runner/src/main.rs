use std::{env, process::exit};

use eval::eval;
use parser::parse;
use typechecker::typecheck;

fn main() {
    env_logger::init();
    match env::args().collect::<Vec<_>>().get(1) {
        Some(path) => {
            println!("Trying to open {}.", path);
            match std::fs::read_to_string(path) {
                Ok(source) => {
                    let p = parse(&source).unwrap();
                    match typecheck(&p) {
                        Ok(_) => {},
                        Err(err) => {
                            eprintln!("Typing Error: {}", err);
                            exit(-1)

                        },
                    }
                    match eval(p) {
                        Ok(()) => {}
                        Err(err) => {
                            eprintln!("Runtime Error: {}", err);
                            exit(-1)
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Ran into error while trying to open the file.");
                    eprintln!("{}", err);
                }
            }
        }
        None => {
            println!("Please provide the path to a file.");
            exit(1);
        }
    };
}
