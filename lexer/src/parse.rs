use crate::ast::{AstNode, Expr, Literal, Op, Pattern, Program, Type};
use crate::info_parse;
use log::info;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./CFG.pest"]
struct LexicalHaskell;

pub fn build_ast(source: &str) -> Result<Program, pest::error::Error<Rule>> {
    let pairs = LexicalHaskell::parse(Rule::program, source)?;
    info!("Found {} decls", pairs.len());
    let res = pairs.map(|p| parse_decl(p)).flatten().collect();
    Ok(res)
}

fn parse_decl(decl: Pair<Rule>) -> Option<AstNode> {
    info_parse!("Declaration", &decl);
    let res = match decl.as_rule() {
        Rule::type_decl => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next()?)?;
            let typ = parse_type(inner.next()?)?;
            Some(AstNode::TypeSignature(var, typ))
        }
        Rule::type_alias => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next()?)?;
            let typ = parse_type(inner.next()?)?;
            Some(AstNode::TypeAlias(var, typ))
        }
        Rule::func_decl => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next()?)?;
            let patterns = parse_patterns(inner.next()?)?;
            let expr = parse_expr(inner.next()?)?;
            Some(AstNode::Decl(var, patterns, expr))
        }
        _ => None,
    };
    info!("Returning {:?}", &res);
    res
}

fn parse_expr(expr: Pair<Rule>) -> Option<Expr> {
    info_parse!("Expression", &expr);
    return match expr.as_rule() {
        Rule::infixop => {
            let mut inner = expr.into_inner();
            let e1 = parse_expr(inner.next()?)?;
            let binop = parse_binop(inner.next()?)?;
            let e2 = parse_expr(inner.next()?)?;
            Some(Expr::BinOp(Box::new(e1), binop, Box::new(e2)))
        }
        Rule::application => {
            let mut inner = expr.into_inner();
            let func_name = parse_symname(inner.next()?)?;
            let args = inner.map(|p| parse_expr(p)).flatten().collect();

            Some(Expr::Application(func_name, args))
        }
        Rule::paren_expr => {
            let mut inner = expr.into_inner();
            let e = parse_expr(inner.next()?)?;
            Some(e)
        }
        Rule::number | Rule::char | Rule::bool | Rule::string => {
            parse_literal(expr).map(Expr::Literal)
        }
        Rule::var_name => parse_symname(expr).map(Expr::Var),
        Rule::tuple_expr => {
            let inner = expr.into_inner();
            let es: Vec<Expr> = inner.map(|p| parse_expr(p)).flatten().collect();
            Some(Expr::Tuple(es))
        }
        Rule::cond => {
            let inner = expr.into_inner();
            let mut es = inner.map(parse_expr).flatten();
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

fn parse_patterns(patterns: Pair<Rule>) -> Option<Vec<Pattern>> {
    let inner = patterns.into_inner();
    let pats = inner.map(parse_pattern).flatten().collect();
    Some(pats)
}

fn parse_pattern(pattern: Pair<Rule>) -> Option<Pattern> {
    info_parse!("Pattern", &pattern);
    return match pattern.as_rule() {
        Rule::number | Rule::char | Rule::bool | Rule::string => {
            Some(Pattern::Literal(parse_literal(pattern)?))
        }
        Rule::var_name => parse_symname(pattern).map(Pattern::Var),
        _ => None,
    };
}

fn parse_literal(literal: Pair<Rule>) -> Option<Literal> {
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
