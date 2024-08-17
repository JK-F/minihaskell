use error::TypeCheckingError;
use parser::ast::AstNode;

pub mod error;
pub mod typecheck;

type TCResult<T> = Result<T, TypeCheckingError>;

pub fn typecheck(program: &Vec<AstNode>) -> TCResult<()> {
    typecheck::type_check(program)
}
