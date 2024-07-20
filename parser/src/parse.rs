use crate::ast::{AstNode, Expr, Op, Pattern, Type, Value};
use crate::error::ParsingError;
use crate::error::ParsingError::GrammarError;
use crate::info_parse;
use log::info;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./CFG.pest"]
struct LexicalHaskell;

pub fn build_ast(source: String) -> Result<Vec<AstNode>, ParsingError> {
    let pairs = LexicalHaskell::parse(Rule::program, &source)?;
    info!("Found {} decls", pairs.len());
    let mut ast = vec![];
    for pair in pairs {
        ast.push(parse_decl(pair)?);
    }
    Ok(ast)
}

fn parse_decl(decl: Pair<Rule>) -> Result<AstNode, ParsingError> {
    info_parse!("Declaration", decl);
    let res = match decl.as_rule() {
        Rule::type_decl => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next().ok_or(GrammarError)?)?;
            let typ = parse_type(inner.next().ok_or(GrammarError)?)?;
            Ok(AstNode::TypeSignature(var, typ))
        }
        Rule::type_alias => {
            let mut inner = decl.into_inner();
            let var = parse_symname(inner.next().ok_or(GrammarError)?)?;
            let typ = parse_type(inner.next().ok_or(GrammarError)?)?;
            Ok(AstNode::TypeAlias(var, typ))
        }
        Rule::func_decl => {
            let mut inner = decl.into_inner();
            // We knwo that there must be a symname next based on the rule being a func_decl
            let fun_name = parse_symname(inner.next().ok_or(GrammarError)?)?;
            let mut fun_rhs = vec![];

            while inner.peek().is_some() {
                let mut debrujin = vec![];
                let patterns = parse_patterns(inner.next().ok_or(GrammarError)?, &mut debrujin)?;
                let expr = parse_expr(inner.next().ok_or(GrammarError)?, &mut debrujin)?;

                if !patterns.is_empty() {
                    merge_decls(fun_name.clone(), &mut fun_rhs, patterns, expr)?;
                } else if inner.peek().is_none() {
                    return Ok(AstNode::Decl(fun_name, Expr::Value(Value::ConstantFunction(Box::new(expr)))));
                } else {
                    // There are no patterns for this function declaratation ==> This is a constant
                    // There is more function declarations within this inner ==> There are multiple
                    // definitions of a constant
                    return Err(ParsingError::MultipleDefinitions(fun_name));
                }

            }

            Ok(AstNode::Decl(fun_name, Expr::Value(Value::Function(fun_rhs))))
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
        | Rule::cond
        | Rule::var_name => {
            let expr = parse_expr(decl, &mut vec![])?;
            Ok(AstNode::SExpr(expr))
        }
        Rule::EOI => Ok(AstNode::EndOfInstruction),
        _ => Err(GrammarError),
    };
    info!("Returning {:?}", &res);
    res
}

fn merge_decls(fun_name: String, fun_rhs: &mut Vec<(Pattern, Expr)>, patterns: Vec<Pattern>, expr: Expr) -> Result<(), ParsingError> {
    assert!(patterns.len() > 0);
    let mut patterns = patterns;
    let pattern = patterns.remove(0);
    let x = fun_rhs.iter_mut().find(|(p, _)| p.eq(&pattern));
    return match x {
        Some((_, Expr::Value(Value::Function(inner_vec)))) => {
            if patterns.len() <= 1 {
                return Err(ParsingError::VaryingArity(fun_name));
            }
            merge_decls(fun_name, inner_vec, patterns, expr)
        }
        Some((_, _)) => {
            if patterns.len() > 1 {
                return Err(ParsingError::VaryingArity(fun_name));
            }
            fun_rhs.push((pattern, expr));
            Ok(())
        }
        None => {
            fun_rhs.push((pattern, expr));
            Ok(())
        },
    };
}

fn parse_expr(expr: Pair<Rule>, debrujin: &mut Vec<String>) -> Result<Expr, ParsingError> {
    info_parse!("Expression", expr);
    let expr = match expr.as_rule() {
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
            let args = inner.map(|p| parse_expr(p, debrujin)).flatten();
            let var = Box::new(Expr::Symbol(func_name));
            let expr: Expr = *args.fold(var, |exp, arg| {
                Box::new(Expr::Application(exp, Box::new(arg)))
            });
            Ok(expr)
        }
        Rule::paren_expr => {
            let mut inner = expr.into_inner();
            let e = parse_expr(inner.next().ok_or(GrammarError)?, debrujin)?;
            Ok(e)
        }
        Rule::number | Rule::char | Rule::bool | Rule::string => {
            parse_literal(expr).map(Expr::Value)
        }
        Rule::var_name => {
            let var = parse_symname(expr)?;
            let index = debrujin
                .iter()
                .rev()
                .position(|v| v.eq(&var))
                .map_or(Expr::Symbol(var), |idx| Expr::Var(idx));
            Ok(index)
        }
        Rule::tuple_expr => {
            let inner = expr.into_inner();
            let es: Vec<Expr> = inner.map(|p| parse_expr(p, debrujin)).flatten().collect();
            Ok(Expr::Tuple(es))
        }
        Rule::cond => {
            let inner = expr.into_inner();
            let mut es: Vec<Expr> = inner.map(|e| parse_expr(e, debrujin)).flatten().collect();
            let else_expr = Box::new(es.pop().ok_or(GrammarError)?);
            let then_expr = Box::new(es.pop().ok_or(GrammarError)?);
            let test = Box::new(es.pop().ok_or(GrammarError)?);
            Ok(Expr::If(
                test,
                then_expr,
                else_expr,
            ))
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

fn parse_patterns(
    patterns: Pair<Rule>,
    debrujin: &mut Vec<String>,
) -> Result<Vec<Pattern>, ParsingError> {
    info_parse!("Patterns", patterns);
    let inner = patterns.into_inner();
    info!("Im here 1");
    let pats = inner
        .map(|pattern| parse_pattern(pattern, debrujin))
        .flatten()
        .collect();
    info!("Im here 2");
    Ok(pats)
}

fn parse_pattern(pattern: Pair<Rule>, debrujin: &mut Vec<String>) -> Result<Pattern, ParsingError> {
    info_parse!("Pattern", pattern);
    return match pattern.as_rule() {
        Rule::number | Rule::char | Rule::bool | Rule::string => {
            debrujin.push(String::new()); // Push for alignment
            Ok(Pattern::Value(parse_literal(pattern)?))
        }
        Rule::var_name => {
            let var = parse_symname(pattern)?;
            debrujin.push(var);
            Ok(Pattern::Var)
        }
        _ => Err(GrammarError),
    };
}

fn parse_literal(literal: Pair<Rule>) -> Result<Value, ParsingError> {
    info_parse!("Literal", literal);
    return match literal.as_rule() {
        Rule::number => {
            let num = literal.as_str();
            if let Ok(val) = num.parse() {
                return Ok(Value::Int(val));
            }
            Err(GrammarError)
        }
        Rule::char => {
            let char = literal.as_str().chars().nth(1).ok_or(GrammarError)?;
            Ok(Value::Char(char))
        }
        Rule::bool => {
            let boolean = literal.as_str();
            if let Ok(val) = boolean.parse() {
                return Ok(Value::Bool(val));
            }
            Err(GrammarError)
        }
        Rule::string => {
            let s = literal.as_str();
            let s = &s[1..s.len() - 1]; // Remove '"'
            Ok(Value::String(s.to_string()))
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
                "Int" => Type::Int,
                "Bool" => Type::Bool,
                "Char" => Type::Char,
                "String" => Type::String,
                _ => Type::TypeName(name)
            })
        },
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

fn parse_symname(name: Pair<Rule>) -> Result<String, ParsingError> {
    info_parse!("Symbol name", name);
    let name = name.as_str();
    return Ok(name.to_string());
}
