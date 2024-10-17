use ast::ast::Program;
use error::TypingError;
use typecheck::typecheck_program;

mod typecheck;
mod error;

pub fn typecheck(p: &Program) -> Result<(), TypingError> {
    let _ = typecheck_program(p)?;
    Ok(())
}
