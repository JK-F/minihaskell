use ast::ast::Type;

use crate::subst::Substitution;
use crate::typecheck::TypeScheme;
use crate::typecheck::TypingEnvironment;

static mut LABEL_COUNTER: i32 = 0;

pub fn fresh_name() -> String {
    let res = format!("<{}>", unsafe { LABEL_COUNTER });
    unsafe {
        LABEL_COUNTER += 1;
    }
    res
}

pub fn scvs_in_type_signature(t: &Type) -> Vec<String> {
    tvars_in(t)
        .into_iter()
        .filter(|name| name.chars().next().unwrap().is_lowercase())
        .cloned()
        .collect()
}

pub fn scvs_given_te(t: &Type, type_env: &TypingEnvironment) -> Vec<String> {
    let unknowns = unknowns_te(type_env);
    let tvars = tvars_in(t);
    tvars
        .into_iter()
        .filter(|tvar| unknowns.contains(tvar))
        .cloned()
        .collect()
}

fn unknowns_te(type_env: &TypingEnvironment) -> Vec<&String> {
    type_env
        .into_iter()
        .flat_map(|(_, scheme)| unknowns_scheme(scheme))
        .collect()
}

fn unknowns_scheme(scheme: &TypeScheme) -> Vec<&String> {
    let (scvs, typ) = scheme;
    let tvars = tvars_in(typ);
    tvars
        .into_iter()
        .filter(|tvar| !scvs.contains(tvar))
        .collect()
}

pub fn tvars_in(t: &Type) -> Vec<&String> {
    match t {
        Type::TypeVariable(tv) => vec![tv],
        Type::Function(t1, t2) => {
            let mut v1 = tvars_in(t1);
            let mut v2 = tvars_in(t2);
            v1.append(&mut v2);
            v1
        }
        Type::Tuple(ts) => ts.into_iter().flat_map(tvars_in).collect(),
        Type::List(t) => tvars_in(t),
        Type::Int | Type::Bool | Type::Char | Type::String => vec![],
    }
}

pub fn sub_type_env(subst: &Substitution, type_env: &TypingEnvironment) -> TypingEnvironment {
    type_env
        .into_iter()
        .map(|(x, scheme_type)| (x.clone(), sub_scheme(subst, scheme_type)))
        .collect()
}

pub fn sub_scheme(subst: &Substitution, scheme_type: &TypeScheme) -> TypeScheme {
    let (scheme_vars, t) = scheme_type;
    let new_scheme = sub_type(&subst.exclude(&scheme_vars), &t);
    (scheme_vars.to_vec(), new_scheme)
}

pub fn sub_type(subst: &Substitution, t: &Type) -> Type {
    match t {
        Type::TypeVariable(tv_name) => subst.apply(tv_name),
        Type::Function(t1, t2) => {
            Type::Function(Box::new(sub_type(subst, t1)), Box::new(sub_type(subst, t2)))
        }
        Type::Tuple(ts) => Type::Tuple(
            ts.into_iter()
                .map(|tuple_type| sub_type(subst, tuple_type))
                .collect(),
        ),
        Type::Int => Type::Int,
        Type::Bool => Type::Bool,
        Type::Char => Type::Char,
        Type::String => Type::String,
        Type::List(t) => Type::List(Box::new(sub_type(subst, t))),
    }
}

pub fn renamed_scheme_vars(type_env: &TypingEnvironment) -> TypingEnvironment {
    type_env
        .into_iter()
        .map(|(x, (scvs, typ))| {
            let scvs_map = scvs.into_iter().map(|scv| (scv.to_string(), fresh_name()));
            let subst = Substitution::from(
                scvs_map
                    .clone()
                    .map(|(fst, snd)| (fst, Type::TypeVariable(snd)))
                    .collect(),
            );
            let new_scvs: Vec<String> = scvs_map.map(|(_, snd)| snd).collect();
            (x.clone(), (new_scvs, sub_type(&subst, typ)))
        })
        .collect()
}
