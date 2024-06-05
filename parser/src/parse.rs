use std::collections::VecDeque;
use crate::ast::{AstNode, Expr, Literal, Op, Pattern, Type};
use crate::info_parse;
use log::info;
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;


#[derive(Parser)]
#[grammar = "./CFG.pest"]
struct LexicalHaskell;

pub fn build_ast(source: &str) -> Result<VecDeque<AstNode>, Error<Rule>> {
    let pairs = LexicalHaskell::parse(Rule::program, source)?;
    info!("Found {} decls", pairs.len());
    let mut debrujin: Vec<&str> = vec![];
    let res = pairs.map(|p| parse_decl(p, &mut debrujin)).flatten().collect();
    Ok(res)
}
fn parse_decl<'a>(decl: Pair<'a, Rule>, debrujin: &mut Vec<&'a str> ) -> Option<AstNode<'a>> {
    info_parse!("Declaration", &decl);
    let res = match decl.as_rule() {
        Rule::type_decl => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next()?)?;
            debrujin.push(var);
            let typ = parse_type(inner.next()?)?;
            Some(AstNode::TypeSignature(0, typ))
        }
        Rule::type_alias => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next()?)?;
            debrujin.push(var);
            let typ = parse_type(inner.next()?)?;
            Some(AstNode::TypeAlias(0, typ))
        }
        Rule::func_decl => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next()?)?;
            debrujin.push(var);
            let patterns = parse_patterns(inner.next()?, debrujin)?;
            let expr = parse_expr(inner.next()?, debrujin)?;
            let count = patterns.iter().filter(|pattern| matches!(pattern, Pattern::Var(_))).count();
            (0..count).for_each(|_| {debrujin.pop();}); // For variable names
            debrujin.pop(); // For function name
            Some(AstNode::Decl(0, patterns, expr))
        }
        _ => None,
    };
    info!("Returning {:?}", &res);
    res
}

fn parse_expr<'a>(expr: Pair<'a, Rule>, debrujin: &mut Vec<&'a str>) -> Option<Expr<'a>> {
    info_parse!("Expression", &expr);
    return match expr.as_rule() {
        Rule::infixop => {
            let mut inner = expr.into_inner();
            let e1 = parse_expr(inner.next()?, debrujin)?;
            let binop = parse_binop(inner.next()?)?;
            let e2 = parse_expr(inner.next()?, debrujin)?;
            Some(Expr::BinOp(Box::new(e1), binop, Box::new(e2)))
        }
        Rule::application => {
            let mut inner = expr.into_inner();
            let func_name = parse_symname(inner.next()?)?;
            let index = debrujin.iter().rev().position(|v| v.eq(&func_name))?;
            let args = inner.map(|p| parse_expr(p, debrujin)).flatten().collect();
            Some(Expr::Application(index, args))
        }
        Rule::paren_expr => {
            let mut inner = expr.into_inner();
            let e = parse_expr(inner.next()?, debrujin)?;
            Some(e)
        }
        Rule::number | Rule::char | Rule::bool | Rule::string => {
            parse_literal(expr).map(Expr::Literal)
        }
        Rule::var_name => {
            let var = parse_symname(expr)?;
            let index = debrujin.iter().rev().position(|v| v.eq(&var))?;
            Some(Expr::Var(index))
        },
        Rule::tuple_expr => {
            let inner = expr.into_inner();
            let es: Vec<Expr> = inner.map(|p| parse_expr(p, debrujin)).flatten().collect();
            Some(Expr::Tuple(es))
        }
        Rule::cond => {
            let inner = expr.into_inner();
            let mut es = inner.map(|e| parse_expr(e, debrujin)).flatten();
            return Some(Expr::If(
                Box::new(es.nth(0)?),
                Box::new(es.nth(1)?),
                Box::new(es.nth(2)?),
            ));
        }
        _ => None,
    };
}

fn parse_binop(infixop: Pair<Rule>) -> Option<Op> {
    info_parse!("Binary Operation", &infixop);
    return match infixop.as_str() {
        "+" => Some(Op::Add),
        "-" => Some(Op::Sub),
        "*" => Some(Op::Mul),
        "/" => Some(Op::Div),
        "==" => Some(Op::Eq),
        "!=" => Some(Op::Neq),
        "<" => Some(Op::Lt),
        ">" => Some(Op::Gt),
        "<=" => Some(Op::Le),
        ">=" => Some(Op::Ge),
        "&&" => Some(Op::And),
        "||" => Some(Op::Or),
        _ => None,
    };
}

fn parse_patterns<'a>(patterns: Pair<'a, Rule>, debrujin: &mut Vec<&'a str>) -> Option<Vec<Pattern<'a>>> {
    let inner = patterns.into_inner();
    let pats = inner.map(|pattern| parse_pattern(pattern, debrujin)).flatten().collect();
    Some(pats)
}

fn parse_pattern<'a>(pattern: Pair<'a, Rule>, debrujin: &mut Vec<&'a str>) -> Option<Pattern<'a>> {
    info_parse!("Pattern", &pattern);
    return match pattern.as_rule() {
        Rule::number | Rule::char | Rule::bool | Rule::string => {
            Some(Pattern::Literal(parse_literal(pattern)?))
        }
        Rule::var_name => {
            let var = parse_symname(pattern)?;
            debrujin.push(var);
            Some(Pattern::Var(0))
        },
        _ => None,
    };
}

fn parse_literal<'a>(literal: Pair<'a, Rule>) -> Option<Literal<'a>> {
    info_parse!("Literal", &literal);
    return match literal.as_rule() {
        Rule::number => {
            let num = literal.as_str();
            if let Ok(val) = num.parse() {
                return Some(Literal::Int(val));
            }
            None
        }
        Rule::char => {
            let char = literal.as_str().chars().nth(1)?;
            Some(Literal::Char(char))
        }
        Rule::bool => {
            let boolean = literal.as_str();
            if let Ok(val) = boolean.parse() {
                return Some(Literal::Bool(val));
            }
            None
        }
        Rule::string => {
            let mut inner = literal.into_inner();
            Some(Literal::String(inner.next()?.as_str()))
        }
        _ => None,
    };
}

fn parse_type(atype: Pair<Rule>) -> Option<Type> {
    info_parse!("Type", &atype);
    return match atype.as_rule() {
        Rule::type_name => parse_symname(atype).map(|x| Type::TypeName(x)),
        Rule::func_type => {
            let mut inner = atype.into_inner();
            let t1 = parse_type(inner.next()?)?;
            let t2 = parse_type(inner.next()?)?;
            Some(Type::Function(Box::new(t1), Box::new(t2)))
        }
        Rule::paren_type => {
            let mut inner = atype.into_inner();
            let t = parse_type(inner.next()?)?;
            Some(t)
        }
        Rule::tuple_type => {
            let inner = atype.into_inner();
            let es: Vec<Type> = inner.map(|p| parse_type(p)).flatten().collect();
            Some(Type::Tuple(es))
        }
        _ => None,
    };
}

fn parse_symname(name: Pair<Rule>) -> Option<&str> {
    info_parse!("Symbol name", &name);
    let name = name.as_str();
    return Some(name);
}
