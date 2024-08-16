use eval::eval;
use parser::parse;


fn main() {
    env_logger::init();
    let source = include_str!("./fib.hs");
    let p = parse(source).unwrap();
    eval(p).unwrap();
}
