use ast::ast::Type;


static mut LABEL_COUNTER: i32 = 0; 

pub fn fresh_name() -> String {
    let res = format!("<{}>", unsafe {LABEL_COUNTER} );
    unsafe {LABEL_COUNTER += 1;}
    res
}

pub fn tvars_in(t: &Type) -> Vec<&String> {
    match t {
        Type::TypeVariable(tv) => vec![tv],
        Type::Function(t1, t2) => {
            let mut v1 = tvars_in(t1);
            let mut v2 = tvars_in(t2);
            v1.append(&mut v2);
            v1
        },
        Type::Tuple(ts) => ts.into_iter().flat_map(tvars_in).collect(),
        Type::List(t) => tvars_in(t),
        Type::Int | Type::Bool | Type::Char | Type::String => vec![],
    }
}

