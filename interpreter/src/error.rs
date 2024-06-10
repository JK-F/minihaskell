use thiserror::Error;

#[derive(Error, Debug)]
pub enum RunTimeError {
    #[error("No variable in scope")]
    OutOfScope,
    #[error("Reached end of program")]
    EndOfProgram,   
}
