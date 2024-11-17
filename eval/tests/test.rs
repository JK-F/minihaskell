use eval::eval;
use parser::parse;

#[test]
fn fib() {
    let _ = env_logger::try_init();
    let src = include_str!("./files/fib.hs");
    let ast = parse(src).unwrap();
    eval(ast).unwrap()
}

#[test]
fn lazy_ignore_inf() {
    let _ = env_logger::try_init();
    let src = include_str!("./files/ignore_inf.hs");
    let ast = parse(src).unwrap();
    eval(ast).unwrap()
}

#[test]
fn infinite_ranges() {
    let _ = env_logger::try_init();
    let src = include_str!("./files/infinite_ranges.hs");
    let ast = parse(src).unwrap();
    eval(ast).unwrap()
}
