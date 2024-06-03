use lexer::parse;
use lexer::test_parse;
use log::info;

#[test]
fn test_literals() {
    test_parse!("./files/literals.hs");
}

#[test]
fn test_fun() {
    test_parse!("./files/fun.hs");
}

#[test]
fn test_complex_fun() {
    test_parse!("./files/complex_fun.hs");
}

#[test]
fn test_var() {
    test_parse!("./files/var.hs");
}

#[test]
fn test_tuples() {
    test_parse!("./files/tuples.hs");
}
