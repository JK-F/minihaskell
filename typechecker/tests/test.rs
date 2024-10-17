use typechecker::typecheck;

#[test]
fn typing_test() {
    env_logger::init();
    let src = include_str!("./files/types.hs");
    let p = parser::parse(src).unwrap();
    let _ = typecheck(&p).unwrap();
}

