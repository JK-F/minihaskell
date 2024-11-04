use ast::ast::Decl;
use ast::ast::Expr;
use log::info;
use parser::parse;

macro_rules! test_parse {
    ($($name:ident: $file:expr,)*) => {
    $(
        #[test]
        fn $name(){
            let _ = env_logger::try_init();
            let src = include_str!($file);
            info!("Reading file: {:?}", $file);
            let ast = parse(src);
            if ast.is_err() {
                info!("{:?}", ast);
            }
            assert!(ast.is_ok());
            let ast = ast.unwrap();
            for decl in ast {
                info!("{:?}", decl);
            }
        }
    )*
    }
}

test_parse! {
    var: "files/var.hs",
    simple_fun: "files/simple_fun.hs",
    complex_fun: "files/complex_fun.hs",
    literals: "files/literals.hs",
    tuples: "files/tuples.hs",
    ifthenelse: "files/ifthen.hs",
    application: "files/appl.hs",
    fib: "files/fib.hs",
    just_exp: "files/just_exp.hs",
    appl_exp: "files/appl_exp.hs",
    pattern: "files/pattern.hs",
    list: "files/list.hs",
    append: "files/append.hs",
    ranges: "files/ranges.hs",
    letin: "files/let.hs",
    lambda: "files/lambda.hs",
}

#[test]
fn application_multiple_args_test() {
    let _ = env_logger::try_init();
    let appl = "f x y\n";
    let program = parse(appl).unwrap();
    let appl = program.first().unwrap().clone();
    let f = Box::new(Expr::Var("f".to_string()));
    let x = Box::new(Expr::Var("x".to_string()));
    let y = Box::new(Expr::Var("y".to_string()));
    assert_eq!(appl, Decl::SExpr(Expr::Application(Box::new(Expr::Application(f, x)), y)));
    
}
