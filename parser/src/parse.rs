use std::collections::HashMap;

use crate::error::ParsingError;
use crate::error::ParsingError::GrammarError;
use crate::info_parse;
use crate::util::gen_arg_name;
use ast::ast::{Decl, Expr, List, Literal, Op, Pattern, Program, Type};
use log::info;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./grammar.pest"]
struct LexicalHaskell;

pub fn build_ast(source: String) -> Result<Program, ParsingError> {
    let pairs = LexicalHaskell::parse(Rule::program, &source)?;
    info!("Found {} decls", pairs.len());
    let mut ast = vec![];
    for pair in pairs {
        ast.push(parse_decl(pair)?);
    }
    Ok(ast)
}

fn parse_decl(decl: Pair<Rule>) -> Result<Decl, ParsingError> {
    info_parse!("Declaration", decl);
    let res = match decl.as_rule() {
        Rule::type_decl => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next().ok_or(GrammarError)?)?;
            let typ = parse_type(inner.next().ok_or(GrammarError)?)?;
            Ok(Decl::TypeSignature(var, typ))
        }
        Rule::type_alias => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next().ok_or(GrammarError)?)?;
            let typ = parse_type(inner.next().ok_or(GrammarError)?)?;
            Ok(Decl::TypeAlias(var, typ))
        }
        Rule::func_decl => {
            let mut inner = decl.into_inner();
            // We knwo that there must be a symname next based on the rule being a func_decl
            let fun_name = parse_symname(inner.next().ok_or(GrammarError)?)?;
            let mut cases = vec![];

            while inner.peek().is_some() {
                let patterns = parse_patterns(inner.next().ok_or(GrammarError)?)?;
                let expr = parse_expr(inner.next().ok_or(GrammarError)?)?;
                let pattern = match &patterns[..] {
                    [] => None,
                    [p] => Some(p.clone()),
                    ps => Some(Pattern::FakeTuple(ps.to_vec())),
                };
                cases.push((pattern, expr));
            }
            let cases = rename_cases(fun_name.clone(), cases);
            return match &cases[..] {
                [] => unreachable!(),
                [(None, e)] => Ok(Decl::FunDecl(fun_name, vec![], e.clone())),
                [(Some(Pattern::Var(name)), e)] => {
                    Ok(Decl::FunDecl(fun_name, vec![name.clone()], e.clone()))
                }
                [(None, _), ..] => Err(ParsingError::MultipleDefinitions(fun_name)),
                [(Some(Pattern::Tuple(_)), _), ..] => {
                    let name = gen_arg_name(fun_name.clone(), 0);
                    let cases = cases.into_iter().map(|(a, b)| (a.unwrap(), b)).collect();
                    let fun_rhs = Expr::Case(Box::new(Expr::Var(name.clone())), cases);
                    Ok(Decl::FunDecl(fun_name, vec![name], fun_rhs))
                }
                [(Some(Pattern::FakeTuple(ps)), _), ..] => {
                    let cloned = fun_name.clone();
                    let args = (0..ps.len())
                        .into_iter()
                        .map(|i| gen_arg_name(cloned.clone(), i));
                    let es = args.clone().map(|name| Expr::Var(name)).collect();
                    let cases = cases.into_iter().map(|(a, b)| (a.unwrap(), b)).collect();
                    let fun_rhs = Expr::Case(Box::new(Expr::Tuple(es)), cases);
                    Ok(Decl::FunDecl(fun_name, args.rev().collect(), fun_rhs))
                }
                _ => {
                    let name = format!("{}:arg", fun_name.clone());
                    let cases = cases.into_iter().map(|(a, b)| (a.unwrap(), b)).collect();
                    let fun_rhs = Expr::Case(Box::new(Expr::Var(name.clone())), cases);
                    Ok(Decl::FunDecl(fun_name, vec![name], fun_rhs))
                }
            };
        }
        Rule::infixop
        | Rule::number
        | Rule::char
        | Rule::bool
        | Rule::string
        | Rule::aexpr
        | Rule::application
        | Rule::paren_expr
        | Rule::tuple_expr
        | Rule::list_expr
        | Rule::cond
        | Rule::var_name => {
            let expr = parse_expr(decl)?;
            Ok(Decl::SExpr(expr))
        }
        Rule::EOI => Ok(Decl::EndOfInstruction),
        _ => Err(GrammarError),
    };
    info!("Returning {:?}", &res);
    res
}

fn parse_expr(expr: Pair<Rule>) -> Result<Expr, ParsingError> {
    info_parse!("Expression", expr);
    let expr = match expr.as_rule() {
        Rule::infixop => {
            let mut inner = expr.into_inner();
            let e1 = parse_expr(inner.next().ok_or(GrammarError)?)?;
            let binop = parse_binop(inner.next().ok_or(GrammarError)?)?;
            let e2 = parse_expr(inner.next().ok_or(GrammarError)?)?;
            Ok(Expr::BinOp(Box::new(e1), binop, Box::new(e2)))
        }
        Rule::application => {
            let inner = expr.into_inner();
            let exprs = inner.map(|p| parse_expr(p)).collect::<Result<Vec<Expr>, _>>()?;
            let expr = exprs.into_iter()
                .reduce(|acc, arg| Expr::Application(Box::new(acc), Box::new(arg)))
                .unwrap();
            Ok(expr)
            // f x y
            // f, x
            // App(f, x)
            // App(f, x), y
            // App(App(f, x), y)
        }
        Rule::paren_expr => {
            let mut inner = expr.into_inner();
            let e = parse_expr(inner.next().ok_or(GrammarError)?)?;
            Ok(e)
        }
        Rule::number | Rule::char | Rule::bool | Rule::string => {
            Ok(Expr::Literal(parse_literal(expr)?))
        }
        Rule::var_name => {
            let var = parse_symname(expr)?;
            Ok(Expr::Var(var))
        }
        Rule::tuple_expr => {
            let inner = expr.into_inner();
            let es: Vec<Expr> = inner.map(|p| parse_expr(p)).collect::<Result<_, _>>()?;
            Ok(Expr::Tuple(es))
        }
        Rule::list_expr => {
            let inner = expr.into_inner();
            let es: Vec<Expr> = inner
                .map(|p| parse_expr(p))
                .rev()
                .collect::<Result<_, _>>()?;
            let list = es
                .into_iter()
                .fold(List::Empty, |acc, e| List::Some(Box::new(e), Box::new(acc)));
            Ok(Expr::List(list))
        }
        Rule::range => {
            let mut inner = expr.into_inner();
            let start = parse_expr(inner.next().ok_or(GrammarError)?)?;
            Ok(Expr::Range(Box::new(start), 1, None))
        }
        Rule::empty_list => Ok(Expr::List(List::Empty)),
        Rule::cond => {
            let inner = expr.into_inner();
            let mut es: Vec<Expr> = inner.map(|e| parse_expr(e)).collect::<Result<_, _>>()?;
            let else_expr = Box::new(es.pop().ok_or(GrammarError)?);
            let then_expr = Box::new(es.pop().ok_or(GrammarError)?);
            let test = Box::new(es.pop().ok_or(GrammarError)?);
            Ok(Expr::If(test, then_expr, else_expr))
        }
        _ => Err(GrammarError),
    };
    return expr;
}

fn parse_binop(infixop: Pair<Rule>) -> Result<Op, ParsingError> {
    info_parse!("Binary Operation", infixop);
    return match infixop.as_str() {
        "+" => Ok(Op::Add),
        "-" => Ok(Op::Sub),
        "*" => Ok(Op::Mul),
        "/" => Ok(Op::Div),
        "`mod`" => Ok(Op::Mod),
        "==" => Ok(Op::Eq),
        "/=" => Ok(Op::Neq),
        "<" => Ok(Op::Lt),
        ">" => Ok(Op::Gt),
        "<=" => Ok(Op::Le),
        ">=" => Ok(Op::Ge),
        "&&" => Ok(Op::And),
        "||" => Ok(Op::Or),
        "++" => Ok(Op::Append),
        ":" => Ok(Op::Cons),
        _ => Err(GrammarError),
    };
}

fn parse_patterns(patterns: Pair<Rule>) -> Result<Vec<Pattern>, ParsingError> {
    info_parse!("Patterns", patterns);
    let inner = patterns.into_inner();
    let pats = inner.map(|pattern| parse_pattern(pattern)).collect();
    pats
}

fn parse_pattern(pattern: Pair<Rule>) -> Result<Pattern, ParsingError> {
    return match pattern.as_rule() {
        Rule::wildcard => Ok(Pattern::Wildcard),
        Rule::empty_list => Ok(Pattern::EmptyList),
        Rule::number | Rule::char | Rule::bool | Rule::string => {
            Ok(Pattern::Literal(parse_literal(pattern)?))
        }
        Rule::var_name => {
            let name = parse_symname(pattern)?;
            Ok(Pattern::Var(name))
        }
        Rule::list_pattern => {
            let mut inner = pattern.into_inner();
            let len = inner.len();
            let head = inner.next().ok_or(ParsingError::GrammarError)?;
            if len == 1 {
                return parse_pattern(head);
            }
            let p1 = parse_pattern(head)?;
            let tail = inner.next().ok_or(ParsingError::GrammarError)?;
            let p2 = parse_pattern(tail)?;
            Ok(Pattern::List(Box::new(p1), Box::new(p2)))
        }
        Rule::tuple_pattern => {
            let inner = pattern.into_inner();
            let ps: Vec<Pattern> = inner.map(|p| parse_pattern(p)).collect::<Result<_, _>>()?;
            Ok(Pattern::Tuple(ps))
        }
        _ => Err(GrammarError),
    };
}

fn parse_literal(literal: Pair<Rule>) -> Result<Literal, ParsingError> {
    info_parse!("Literal", literal);
    return match literal.as_rule() {
        Rule::number => {
            let num = literal.as_str();
            if let Ok(val) = num.parse() {
                return Ok(Literal::Int(val));
            }
            Err(GrammarError)
        }
        Rule::char => {
            let char = literal.as_str().chars().nth(1).ok_or(GrammarError)?;
            Ok(Literal::Char(char))
        }
        Rule::bool => {
            let boolean = literal.as_str();
            match boolean {
                "True" => Ok(Literal::Bool(true)),
                "False" => Ok(Literal::Bool(false)),
                _ => Err(GrammarError),
            }
        }
        Rule::string => {
            let s = literal.as_str();
            let s = &s[1..s.len() - 1]; // Remove '"'
            Ok(Literal::String(s.to_string()))
        }
        _ => Err(GrammarError),
    };
}

fn parse_type(atype: Pair<Rule>) -> Result<Type, ParsingError> {
    info_parse!("Type", atype);
    return match atype.as_rule() {
        Rule::type_name => {
            let name = parse_symname(atype)?;
            Ok(match name.as_str() {
                "Int" | "Integer" => Type::Int,
                "Bool" => Type::Bool,
                "Char" => Type::Char,
                "String" => Type::String,
                _ => Type::TypeVariable(name),
            })
        }
        Rule::func_type => {
            let mut inner = atype.into_inner();
            let t1 = parse_type(inner.next().ok_or(GrammarError)?)?;
            let t2 = parse_type(inner.next().ok_or(GrammarError)?)?;
            Ok(Type::Function(Box::new(t1), Box::new(t2)))
        }
        Rule::paren_type => {
            let mut inner = atype.into_inner();
            let t = parse_type(inner.next().ok_or(GrammarError)?)?;
            Ok(t)
        }
        Rule::tuple_type => {
            let inner = atype.into_inner();
            let es: Vec<Type> = inner.map(|p| parse_type(p)).collect::<Result<_, _>>()?;
            Ok(Type::Tuple(es))
        }
        _ => Err(GrammarError),
    };
}

fn parse_symname(name: Pair<Rule>) -> Result<String, ParsingError> {
    info_parse!("Symbol name", name);
    let name = name.as_str();
    return Ok(name.to_string());
}

fn rename_cases(
    fun_name: String,
    cases: Vec<(Option<Pattern>, Expr)>,
) -> Vec<(Option<Pattern>, Expr)> {
    cases
        .into_iter()
        .map(|case| merge_var_bindings(fun_name.clone(), case))
        .collect()
}

fn merge_var_bindings(fun_name: String, case: (Option<Pattern>, Expr)) -> (Option<Pattern>, Expr) {
    let (pattern, expr) = case;
    match pattern {
        Some(Pattern::FakeTuple(ps)) => {
            info!("Merging var names on {}", ps.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" "));
            let renamings = ps
                .iter()
                .enumerate()
                .map(|(i, p)| (gen_arg_name(fun_name.clone(), i), p))
                .map(|(arg_name, pattern)| match pattern {
                    Pattern::Var(var_name) => Some((var_name.clone(), arg_name)),
                    _ => None,
                })
                .flatten()
                .collect();
            let expr = rename_vars(&renamings, expr);
            let pattern = Pattern::FakeTuple(
                ps.into_iter()
                    .map(|p| match p {
                        Pattern::Var(name) => Pattern::Var(renamings.get(&name).unwrap().clone()),
                        some_p => some_p,
                    })
                    .collect(),
            );
            (Some(pattern), expr)
        }
        case => (case, expr),
    }
}

fn rename_vars(renamings: &HashMap<String, String>, expr: Expr) -> Expr {
    renamings
        .iter()
        .fold(expr, |acc, (old, new)| rename_expr(acc, old, new))
}

fn rename_expr(expr: Expr, old: &String, new: &String) -> Expr {
    match expr {
        Expr::Var(name) => {
            return if name.eq(old) {
                Expr::Var(new.clone())
            } else {
                Expr::Var(name)
            }
        }
        Expr::Application(f, e) => Expr::Application(
            Box::new(rename_expr(*f, old, new)),
            Box::new(rename_expr(*e, old, new)),
        ),
        Expr::If(a, b, c) => Expr::If(
            Box::new(rename_expr(*a, old, new)),
            Box::new(rename_expr(*b, old, new)),
             Box::new(rename_expr(*c, old, new)),
        ),
        Expr::Case(e, cases) => Expr::Case(Box::new(rename_expr(*e, old, new)), cases.into_iter().map(|(p, e)| rename_case(p, e, old, new) ).collect()),
        Expr::BinOp(l, op, r) => Expr::BinOp(Box::new(rename_expr(*l, old, new)), op, Box::new(rename_expr(*r, old, new))),
        Expr::Tuple(es) => Expr::Tuple(es.into_iter().map(|e| rename_expr(e, old, new)).collect()),
        Expr::List(ls) =>  Expr::List(rename_list(ls, old, new)),
        Expr::Literal(l) => Expr::Literal(l),
        Expr::Range(start, step, stop) => Expr::Range(Box::new(rename_expr(*start, old, new)), step, stop),
    }
}

fn rename_list(ls: List<Expr>, old: &String, new: &String) -> List<Expr> {
    match ls {
        List::Some(e, es) => List::Some(Box::new(rename_expr(*e, old, new)), Box::new(rename_list(*es, old, new))),
        List::Empty => List::Empty,
    }
}

fn rename_case(p: Pattern, e: Expr, old: &String, new: &String) -> (Pattern, Expr) {
    if is_bound(&p, &old) {
        return (p, e);
    }
    return (p, rename_expr(e, old, new));
}

fn is_bound(p: &Pattern, old: &String) -> bool {
    match p {
        Pattern::Var(name) => name.eq(old),
        Pattern::Tuple(ps)     => ps.into_iter().any(|p| is_bound(p, old)),
        Pattern::FakeTuple(ps)     => ps.into_iter().any(|p| is_bound(p, old)),
        Pattern::List(p1, p2) => is_bound(p1, old) || is_bound(p2, old),
        Pattern::Literal(_) => false,
        Pattern::Wildcard => false,
        Pattern::EmptyList => false,
    }
}
