use crate::ast::{AstNode, Expr, Literal, Op, Pattern, Type};
use crate::error::ParsingError;
use crate::error::ParsingError::GrammarError;
use crate::info_parse;
use log::info;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::VecDeque;

#[derive(Parser)]
#[grammar = "./CFG.pest"]
struct LexicalHaskell;

pub fn build_ast(source: &str) -> Result<VecDeque<AstNode>, ParsingError> {
    let pairs = LexicalHaskell::parse(Rule::program, source)?;
    info!("Found {} decls", pairs.len());
    let mut debrujin: Vec<&str> = vec![];
    let res = pairs
        .map(|p| parse_decl(p, &mut debrujin))
        .flatten()
        .collect();
    Ok(res)
}

fn parse_decl<'a>(
    decl: Pair<'a, Rule>,
    debrujin: &mut Vec<&'a str>,
) -> Result<AstNode<'a>, ParsingError<'a>> {
    info_parse!("Declaration", &decl);
    let res = match decl.as_rule() {
        Rule::type_decl => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next().ok_or(GrammarError)?)?;
            debrujin.push(var);
            let typ = parse_type(inner.next().ok_or(GrammarError)?)?;
            Ok(AstNode::TypeSignature(0, typ))
        }
        Rule::type_alias => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next().ok_or(GrammarError)?)?;
            debrujin.push(var);
            let typ = parse_type(inner.next().ok_or(GrammarError)?)?;
            Ok(AstNode::TypeAlias(0, typ))
        }
        Rule::func_decl => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next().ok_or(GrammarError)?)?;
            debrujin.push(var);
            let patterns = parse_patterns(inner.next().ok_or(GrammarError)?, debrujin)?;
            let expr = parse_expr(inner.next().ok_or(GrammarError)?, debrujin)?;
            let count = patterns
                .iter()
                .filter(|pattern| matches!(pattern, Pattern::Var(_)))
                .count();
            (0..count).for_each(|_| {
                debrujin.pop();
            }); // For variable names
            debrujin.pop(); // For function name
            Ok(AstNode::Decl(0, patterns, expr))
        }
        Rule::EOI => Ok(AstNode::EndOfInstruction),
        _ => Err(GrammarError),
    };
    info!("Returning {:?}", &res);
    res
}

fn parse_expr<'a>(
    expr: Pair<'a, Rule>,
    debrujin: &mut Vec<&'a str>,
) -> Result<Expr<'a>, ParsingError<'a>> {
    info_parse!("Expression", &expr);
    return match expr.as_rule() {
        Rule::infixop => {
            let mut inner = expr.into_inner();
            let e1 = parse_expr(inner.next().ok_or(GrammarError)?, debrujin)?;
            let binop = parse_binop(inner.next().ok_or(GrammarError)?)?;
            let e2 = parse_expr(inner.next().ok_or(GrammarError)?, debrujin)?;
            Ok(Expr::BinOp(Box::new(e1), binop, Box::new(e2)))
        }
        Rule::application => {
            let mut inner = expr.into_inner();
            let func_name = parse_symname(inner.next().ok_or(GrammarError)?)?;
            info!("Debrujin: {:?}, variable: {:?}", debrujin, func_name);
            let index = debrujin
                .iter()
                .rev()
                .position(|v| v.eq(&func_name))
                .ok_or(ParsingError::UnknownSymbol(func_name))?;
            let args = inner.map(|p| parse_expr(p, debrujin)).flatten().collect();
            Ok(Expr::Application(index, args))
        }
        Rule::paren_expr => {
            let mut inner = expr.into_inner();
            let e = parse_expr(inner.next().ok_or(GrammarError)?, debrujin)?;
            Ok(e)
        }
        Rule::number | Rule::char | Rule::bool | Rule::string => {
            parse_literal(expr).map(Expr::Literal)
        }
        Rule::var_name => {
            let var = parse_symname(expr)?;
            let index = debrujin
                .iter()
                .rev()
                .position(|v| v.eq(&var))
                .ok_or(ParsingError::UnknownSymbol(var))?;
            Ok(Expr::Var(index))
        }
        Rule::tuple_expr => {
            let inner = expr.into_inner();
            let es: Vec<Expr> = inner.map(|p| parse_expr(p, debrujin)).flatten().collect();
            Ok(Expr::Tuple(es))
        }
        Rule::cond => {
            let inner = expr.into_inner();
            let mut es = inner.map(|e| parse_expr(e, debrujin)).flatten();
            return Ok(Expr::If(
                Box::new(es.nth(0).ok_or(GrammarError)?),
                Box::new(es.nth(1).ok_or(GrammarError)?),
                Box::new(es.nth(2).ok_or(GrammarError)?),
            ));
        }
        _ => Err(GrammarError),
    };
}

fn parse_binop(infixop: Pair<Rule>) -> Result<Op, ParsingError> {
    info_parse!("Binary Operation", &infixop);
    return match infixop.as_str() {
        "+" => Ok(Op::Add),
        "-" => Ok(Op::Sub),
        "*" => Ok(Op::Mul),
        "/" => Ok(Op::Div),
        "==" => Ok(Op::Eq),
        "!=" => Ok(Op::Neq),
        "<" => Ok(Op::Lt),
        ">" => Ok(Op::Gt),
        "<=" => Ok(Op::Le),
        ">=" => Ok(Op::Ge),
        "&&" => Ok(Op::And),
        "||" => Ok(Op::Or),
        _ => Err(GrammarError),
    };
}

fn parse_patterns<'a>(
    patterns: Pair<'a, Rule>,
    debrujin: &mut Vec<&'a str>,
) -> Result<Vec<Pattern<'a>>, ParsingError<'a>> {
    info_parse!("Patterns", &patterns);
    let inner = patterns.into_inner();
    let pats = inner
        .map(|pattern| parse_pattern(pattern, debrujin))
        .flatten()
        .collect();
    Ok(pats)
}

fn parse_pattern<'a>(
    pattern: Pair<'a, Rule>,
    debrujin: &mut Vec<&'a str>,
) -> Result<Pattern<'a>, ParsingError<'a>> {
    info_parse!("Pattern", &pattern);
    return match pattern.as_rule() {
        Rule::number | Rule::char | Rule::bool | Rule::string => {
            Ok(Pattern::Literal(parse_literal(pattern)?))
        }
        Rule::var_name => {
            let var = parse_symname(pattern)?;
            debrujin.push(var);
            Ok(Pattern::Var(0))
        }
        _ => Err(GrammarError),
    };
}

fn parse_literal<'a>(literal: Pair<'a, Rule>) -> Result<Literal<'a>, ParsingError<'a>> {
    info_parse!("Literal", &literal);
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
            if let Ok(val) = boolean.parse() {
                return Ok(Literal::Bool(val));
            }
            Err(GrammarError)
        }
        Rule::string => {
            let mut inner = literal.into_inner();
            Ok(Literal::String(inner.next().ok_or(GrammarError)?.as_str()))
        }
        _ => Err(GrammarError),
    };
}

fn parse_type(atype: Pair<Rule>) -> Result<Type, ParsingError> {
    info_parse!("Type", &atype);
    return match atype.as_rule() {
        Rule::type_name => parse_symname(atype).map(|x| Type::TypeName(x)),
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
            let es: Vec<Type> = inner.map(|p| parse_type(p)).flatten().collect();
            Ok(Type::Tuple(es))
        }
        _ => Err(GrammarError),
    };
}

fn parse_symname(name: Pair<Rule>) -> Result<&str, ParsingError> {
    info_parse!("Symbol name", &name);
    let name = name.as_str();
    return Ok(name);
}
