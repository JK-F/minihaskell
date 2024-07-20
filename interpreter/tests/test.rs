use interpreter::interpret;
use parser::parse;

#[test]
fn fib_test() {
    let _ = env_logger::try_init();
    let src = include_str!("./files/fib.hs");
    let ast = parse(src).unwrap();
    interpret(ast);
}
