use log::info;
use parser::parse;
use parser::test_parse;

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
}
