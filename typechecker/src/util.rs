use std::collections::HashSet;

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

pub fn scvs_in_type_signature(t: &Type) -> HashSet<String> {
    tvars_in(t)
        .into_iter()
        .filter(|name| name.chars().next().unwrap().is_lowercase())
        .cloned()
        .collect()
}

pub fn scvs_given_te(t: &Type, type_env: &TypingEnvironment) -> HashSet<String> {
    let unknowns = unknowns_te(type_env);
    let tvars = tvars_in(t);
    tvars
        .into_iter()
        .filter(|tvar| !unknowns.contains(tvar))
        .cloned()
        .collect()
}

fn unknowns_te(type_env: &TypingEnvironment) -> HashSet<&String> {
    type_env
        .into_iter()
        .flat_map(|(_, scheme)| unknowns_scheme(scheme))
        .collect()
}

fn unknowns_scheme(scheme: &TypeScheme) -> HashSet<&String> {
    let (scvs, typ) = scheme;
    let tvars = tvars_in(typ);
    tvars
        .into_iter()
        .filter(|tvar| !scvs.contains(*tvar))
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
    (scheme_vars.clone(), new_scheme)
}

pub fn sub_type(subst: &Substitution, t: &Type) -> Type {
    match t {
        Type::TypeVariable(tv_name) => {
            match subst.apply(tv_name) {
                Type::TypeVariable(new_tv_name) => {
                    if new_tv_name.eq(tv_name) {
                        return Type::TypeVariable(new_tv_name);
                    }
                    sub_type(subst, &Type::TypeVariable(new_tv_name))
                },
                Type::Int => Type::Int,
                Type::Bool => Type::Bool,
                Type::Char => Type::Char,
                Type::String => Type::String,
                new_t => sub_type(subst, &new_t),
            }
        },
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
