use std::{
    collections::{HashMap, HashSet},
    iter::zip,
};

use crate::{
    error::TypingError,
    subst::{subst_combine, Substitution},
    util::{fresh_name, scvs_given_te, scvs_in_type_signature, sub_type, sub_type_env},
};
use ast::ast::{Decl, Expr, List, Literal, Op, Pattern, Program, Type};
use log::info;

pub type TypeScheme = (HashSet<String>, Type);
pub type TypingEnvironment = HashMap<String, TypeScheme>;

pub fn typecheck_program(p: &Program) -> Result<Substitution, TypingError> {
    let mut type_env = TypingEnvironment::new();
    let mut subst = Substitution::id_subst();
    for decl in p {
        subst = typecheck_decl(&mut type_env, subst, decl)?;
    }
    Ok(subst)
}

fn typecheck_decl(
    type_env: &mut TypingEnvironment,
    subst: Substitution,
    decl: &Decl,
) -> Result<Substitution, TypingError> {
    match decl {
        Decl::TypeAlias(var_name, type1) => {
            info!("Introducing type alias {} = {}", var_name, type1);
            subst.extended(var_name.clone(), type1.clone())
        }
        Decl::TypeSignature(var_name, type1) => {
            info!("Introducing type signature {} :: {}", var_name, type1);
            let schematic_vars = scvs_in_type_signature(type1);
            if let Some((scvs2, type2)) =
                type_env.insert(var_name.clone(), (schematic_vars.clone(), type1.clone()))
            {
                let map1 = schematic_vars
                    .into_iter()
                    .map(|name| (name, Type::TypeVariable(fresh_name())))
                    .collect::<HashMap<String, Type>>();
                let map2 = scvs2
                    .into_iter()
                    .map(|name| (name, Type::TypeVariable(fresh_name())))
                    .collect::<HashMap<String, Type>>();
                let subst1 = Substitution::from(map1);
                let subst2 = Substitution::from(map2);
                return unify(subst, &sub_type(&subst1, type1), &sub_type(&subst2, &type2));
            }
            Ok(subst)
        }
        Decl::FunDecl(name, vars, expr) => {
            info!("Type checking {name} with arguments {}", vars.join(", "));
            let mut expr_env = type_env.clone();
            if !expr_env.contains_key(name) {
                let fresh = fresh_name();
                expr_env.insert(name.clone(), (HashSet::new(), Type::TypeVariable(fresh)));
            }
            let arg_renamings = vars
                .into_iter()
                .map(|arg| (arg, fresh_name()))
                .collect::<Vec<_>>();
            for (var, fresh) in arg_renamings.clone() {
                expr_env.insert(var.clone(), (HashSet::new(), Type::TypeVariable(fresh)));
            }
            let (return_subst, return_type) = typecheck_expression(&mut expr_env, expr)?;
            let subst = subst_combine(subst, return_subst);
            let fun_type = arg_renamings
                .into_iter()
                .map(|(_, arg_type)| arg_type)
                .fold(return_type, |acc, arg_type| {
                    Type::Function(Box::new(Type::TypeVariable(arg_type)), Box::new(acc))
                });
            let fun_type = sub_type(&subst, &fun_type);
            let scvs = scvs_given_te(&fun_type, &type_env);
            info!(
                "Resulting in fun_type [{}], {}",
                scvs.iter().cloned().collect::<Vec<_>>().join(", "),
                fun_type
            );
            if let Some((_, saved_type)) = type_env.get(name) {
                return unify(subst, &saved_type, &fun_type);
            } else {
                type_env.insert(name.to_string(), (scvs, fun_type));
            }
            Ok(subst)
        }
        Decl::SExpr(e) => typecheck_expression(type_env, e).map(|(subst, _)| subst),
        Decl::EndOfInstruction => Ok(Substitution::id_subst()),
    }
}

fn typecheck_expression(
    type_env: &mut TypingEnvironment,
    expr: &Expr,
) -> Result<(Substitution, Type), TypingError> {
    info!("Typechecking expr {}", expr);
    info!("With Type Env: {:?}", type_env);
    match expr {
        Expr::Var(x) => {
            let (scheme_vars, old_type) = type_env
                .get(x)
                .ok_or(TypingError::UnknownIdentifier(x.clone()))?;
            let map = scheme_vars
                .into_iter()
                .map(|var| (var.clone(), Type::TypeVariable(fresh_name())))
                .collect::<HashMap<_, _>>();
            let phi = Substitution::from(map);
            let new_type = sub_type(&phi, old_type);

            info!("Giving {x} :: {old_type} new type {new_type}");
            Ok((Substitution::id_subst(), new_type))
        }
        Expr::Application(f, e) => {
            let (phi, type_f) = typecheck_expression(type_env, f)?;
            let mut new_env = sub_type_env(&phi, &type_env);
            let (psi, type_e) = typecheck_expression(&mut new_env, e)?;
            let subst = subst_combine(psi, phi);
            let tv_name = fresh_name();
            let t = Type::TypeVariable(tv_name.clone());
            let subst = unify(
                subst,
                &type_f,
                &Type::Function(Box::new(type_e), Box::new(Type::TypeVariable(tv_name))),
            )?;
            Ok((subst, t))
        }
        Expr::If(cond, then_branch, else_branch) => {
            let (phi, type_cond) = typecheck_expression(type_env, cond)?;
            let (subst_then, type_then) = typecheck_expression(type_env, then_branch)?;
            let (subst_else, type_else) = typecheck_expression(type_env, else_branch)?;
            let cond_subst = unify(phi, &type_cond, &Type::Bool)?;
            let subst_branches = subst_combine(subst_then, subst_else);
            let subst = subst_combine(cond_subst, subst_branches);
            let subst = unify(subst, &type_then, &type_else)?;
            Ok((subst, type_then))
        }
        Expr::Lambda(arg, expr) => {
            let mut type_env = type_env.clone();
            let fresh = fresh_name();
            type_env.insert(
                arg.clone(),
                (HashSet::new(), Type::TypeVariable(fresh.clone())),
            );
            let (subst, ret_type) = typecheck_expression(&mut type_env, expr)?;
            Ok((
                subst,
                Type::Function(Box::new(Type::TypeVariable(fresh)), Box::new(ret_type)),
            ))
        }
        Expr::Let(var, expr1, expr2) => {
            let mut let_env = type_env.clone();
            let (subst1, var_type) = typecheck_expression(&mut let_env, &expr1)?;
            info!("Resulting in {subst1:?}");
            let mut let_env = sub_type_env(&subst1, &let_env);
            info!("For type env {let_env:?}");
            // add decls
            let scvs = scvs_given_te(&var_type, &let_env);
            let mapping = Substitution::from(scvs.into_iter().map(|scv| (scv, Type::TypeVariable(fresh_name()))).collect());
            let new_type = sub_type(&mapping, &var_type);
            let scheme_vars = mapping.range();
            info!("Finding scheme vars {:?}", scheme_vars);
            let _ = let_env.insert(var.clone(), (scheme_vars, new_type));
            // end
            let (subst2, return_type) = typecheck_expression(&mut let_env, &expr2)?;
            Ok((subst_combine(subst2, subst1), return_type))
        }
        Expr::Case(case_expr, cases) => {
            let mut type_env = type_env.clone();
            let (expr_subst, case_expr_type) = typecheck_expression(&mut type_env, case_expr)?;
            let mut subst = expr_subst;
            // Placeholder there cant be an empty case would be stupid
            let mut return_type = None;
            for (pattern, case_body) in cases {
                let (pattern_subst, pattern_type) =
                    typecheck_pattern(&mut type_env, subst, pattern)?;
                subst = unify(pattern_subst, &case_expr_type, &pattern_type)?;
                let (case_subst, case_body_type) = typecheck_expression(&mut type_env, case_body)?;
                subst = subst_combine(case_subst, subst);
                if let Some(body_type) = return_type {
                    subst = unify(subst, &body_type, &case_body_type)?;
                }
                return_type = Some(case_body_type);
            }
            Ok((subst, return_type.unwrap()))
        }
        Expr::BinOp(left, op, right) => {
            let (left_subst, left_type) = typecheck_expression(type_env, left)?;
            let (right_subst, right_type) = typecheck_expression(type_env, right)?;
            let subst = subst_combine(left_subst, right_subst);
            match op {
                Op::Add | Op::Sub | Op::Mul | Op::Mod | Op::Div => {
                    let subst = unify(subst, &left_type, &Type::Int)?;
                    let subst = unify(subst, &right_type, &Type::Int)?;
                    Ok((subst, Type::Int))
                }
                Op::Eq | Op::Neq => {
                    let subst = unify(subst, &left_type, &right_type)?;
                    Ok((subst, Type::Bool))
                },
                Op::Lt | Op::Gt | Op::Le | Op::Ge => {
                    let subst = unify(subst, &left_type, &Type::Int)?;
                    let subst = unify(subst, &left_type, &right_type)?;
                    Ok((subst, Type::Bool))
                }
                Op::And | Op::Or => {
                    let subst = unify(subst, &left_type, &Type::Bool)?;
                    let subst = unify(subst, &left_type, &right_type)?;
                    Ok((subst, Type::Bool))
                }
                Op::Append => {
                    let subst = unify(subst, &left_type, &right_type)?;
                    let subst = unify(subst, &left_type, &right_type)?;
                    Ok((subst, left_type))
                }
                Op::Cons => {
                    let list_type = Type::List(Box::new(left_type));
                    let subst = unify(subst, &list_type, &right_type)?;
                    Ok((subst, list_type))
                }
            }
        }
        Expr::Tuple(exprs) => {
            let mut env = type_env.clone();
            let mut types = vec![];
            let mut subst = Substitution::id_subst();
            for expr in exprs {
                let (expr_subst, typ) = typecheck_expression(&mut env, expr)?;
                env = sub_type_env(&expr_subst, &env);
                subst = subst_combine(expr_subst, subst);
                types.push(typ);
            }
            let tuple_type = Type::Tuple(types.iter().map(|t| sub_type(&subst, &t)).collect());
            Ok((subst, tuple_type))
        }
        Expr::List(es) => match es {
            List::Some(first, tail) => {
                let (subst_first, type_first) = typecheck_expression(type_env, first)?;
                let (subst_tail, type_tail) =
                    typecheck_expression(type_env, &Expr::List(*tail.clone()))?;
                let subst = subst_combine(subst_first, subst_tail);
                let subst = unify(subst, &Type::List(Box::new(type_first.clone())), &type_tail)?;
                Ok((subst, Type::List(Box::new(type_first))))
            }
            List::Empty => Ok((
                Substitution::id_subst(),
                Type::List(Box::new(Type::TypeVariable(fresh_name()))),
            )),
        },
        Expr::Range(from, step, to) => {
            let (subst_from, type_from) = typecheck_expression(type_env, from)?;
            let subst_from = unify(subst_from, &type_from, &Type::Int)?;
            let (subst_step, type_step) = typecheck_expression(type_env, step)?;
            let subst_step = unify(subst_step, &type_step, &Type::Int)?;
            let subst = subst_combine(subst_step, subst_from);
            let subst = match to {
                Some(to) => {
                    let (subst_to, type_to) = typecheck_expression(type_env, to)?;
                    let subst_to = unify(subst_to, &type_to, &Type::Int)?;
                    subst_combine(subst_to, subst)
                }
                None => subst,
            };
            Ok((subst, Type::List(Box::new(Type::Int))))
        }
        Expr::Literal(Literal::Int(_)) => Ok((Substitution::id_subst(), Type::Int)),
        Expr::Literal(Literal::Bool(_)) => Ok((Substitution::id_subst(), Type::Bool)),
        Expr::Literal(Literal::Char(_)) => Ok((Substitution::id_subst(), Type::Char)),
        Expr::Literal(Literal::String(_)) => Ok((Substitution::id_subst(), Type::String)),
    }
}

fn typecheck_pattern(
    type_env: &mut TypingEnvironment,
    subst: Substitution,
    pattern: &Pattern,
) -> Result<(Substitution, Type), TypingError> {
    info!("Type checking pattern {}", pattern);
    match pattern {
        Pattern::Literal(Literal::Int(_)) => Ok((Substitution::id_subst(), Type::Int)),
        Pattern::Literal(Literal::Bool(_)) => Ok((Substitution::id_subst(), Type::Bool)),
        Pattern::Literal(Literal::Char(_)) => Ok((Substitution::id_subst(), Type::Char)),
        Pattern::Literal(Literal::String(_)) => Ok((Substitution::id_subst(), Type::String)),
        Pattern::Var(var_name) => {
            let fresh = fresh_name();
            let type_variable = Type::TypeVariable(fresh.clone());
            type_env.insert(var_name.clone(), (HashSet::new(), type_variable.clone()));
            Ok((Substitution::id_subst(), type_variable))
        }
        Pattern::List(first, tail) => {
            let (subst, first_type) = typecheck_pattern(type_env, subst, first)?;
            let (subst, tail_type) = typecheck_pattern(type_env, subst, tail)?;
            info!("Type checking list pattern ({}:{})", first_type, tail_type);
            let list_type = Type::List(Box::new(first_type.clone()));
            let subst = unify(subst, &list_type, &tail_type)?;
            Ok((subst, list_type))
        }
        Pattern::Tuple(ps) => {
            let mut types = vec![];
            let mut current_subst = subst;
            for pattern in ps {
                let (phi, pattern_type) = typecheck_pattern(type_env, current_subst, pattern)?;
                current_subst = phi;
                types.push(pattern_type);
            }
            Ok((current_subst, Type::Tuple(types)))
        }
        Pattern::FakeTuple(ps) => {
            let mut types = vec![];
            let mut current_subst = subst;
            for pattern in ps {
                let (phi, pattern_type) = typecheck_pattern(type_env, current_subst, pattern)?;
                current_subst = phi;
                types.push(pattern_type);
            }
            Ok((current_subst, Type::Tuple(types)))
        }
        Pattern::EmptyList => Ok((
            Substitution::id_subst(),
            Type::List(Box::new(Type::TypeVariable(fresh_name()))),
        )),
        Pattern::Wildcard => Ok((Substitution::id_subst(), Type::TypeVariable(fresh_name()))),
    }
}

fn unify(phi: Substitution, t1: &Type, t2: &Type) -> Result<Substitution, TypingError> {
    info!("unifying {} and {}", t1, t2);
    match (t1, t2) {
        (Type::TypeVariable(tv_name), t2) | (t2, Type::TypeVariable(tv_name)) => {
            let phit = sub_type(&phi, &t2);
            let phitvn = phi.apply(&tv_name);
            info!("Unification: Translated {} to {}", tv_name, phitvn);
            if phitvn == Type::TypeVariable(tv_name.clone()) {
                return phi.extended(tv_name.clone(), phit);
            }
            unify(phi, &phitvn, &phit)
        }
        (Type::Function(lt1, lt2), Type::Function(rt1, rt2)) => {
            let phi = unify(phi, lt1, rt1)?;
            unify(phi, lt2, rt2)
        }
        (Type::Tuple(lts), Type::Tuple(rts)) => {
            let ts = zip(lts, rts);
            ts.fold(Ok(phi), |acc_phi, (t1, t2)| {
                acc_phi.and_then(|acc_phi| unify(acc_phi, t1, t2))
            })
        }
        (Type::Int, Type::Int)
        | (Type::Bool, Type::Bool)
        | (Type::Char, Type::Char)
        | (Type::String, Type::String) => Ok(phi),
        (Type::List(type1), Type::List(type2)) => unify(phi, type1, type2),
        (x, y) => Err(TypingError::CannotUnify(x.clone(), y.clone())),
    }
}
