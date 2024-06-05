use log::info;
use parser::parse;
use parser::test_parse;


test_parse!{
    var: "files/var.hs",
    fun: "files/fun.hs",
    complex_fun: "files/complex_fun.hs",
    literals: "files/literals.hs",
    tuples: "files/tuples.hs",
    ifthenelse: "files/ifthen.hs",
}
