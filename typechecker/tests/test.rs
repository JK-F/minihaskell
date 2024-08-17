use typechecker::typecheck;

#[test]
fn typing_test() {
    let src = include_str!("./files/types.hs");
    let p = parser::parse(src).unwrap();
    assert!(typecheck(&p).is_ok());
}
