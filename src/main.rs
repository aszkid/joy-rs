extern crate nom;

use nom::IResult;
use nom::combinator::{opt, recognize, map_res, map};
use nom::branch::{alt};
use nom::sequence::{tuple, pair, delimited};
use nom::character::complete::{alpha1, digit1, none_of, one_of};
use nom::bytes::complete::{tag, escaped, take_while};
use nom::multi::{many1, separated_list};

use std::io::{self, Write};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
enum StackElem {
    Number(i32),
    Boolean(bool),
    Quotation(Vec<String>),
    Quoth(Box<Vec<StackElem>>),
    Symbol(String),
    Decimal(f32),
    String(String),
}

#[derive(Debug, Clone)]
enum Token {
    Number(i32),
    Boolean(bool),
    Decimal(f32),
    String(String),
    Symbol(String),
    Quotation(Box<Vec<Token>>)
}

fn maybe_signed_digits(s: &str) -> IResult<&str, &str> {
    recognize(pair(
        opt(alt((tag("+"), tag("-")))),
        digit1
    ))(s)
}

fn floating_point(s: &str) -> IResult<&str, &str> {
    recognize(tuple((
        maybe_signed_digits,
        tag("."),
        digit1
    )))(s)
}

fn decimal(s: &str) -> IResult<&str, Token> {
    map_res(
        floating_point,
        |s| f32::from_str(s).and_then(|f| Ok(Token::Decimal(f)))
    )(s)
}

fn number(s: &str) -> IResult<&str, Token> {
    map_res(
        maybe_signed_digits,
        |s| i32::from_str(s).and_then(|n| Ok(Token::Number(n)))
    )(s)
}

fn literal_in(s: &str) -> IResult<&str, &str> {
    alt((
        delimited(
            tag("'"),
            alt((
                escaped(none_of("\\\'"), '\\', tag("'")),
                tag("")
            )),
            tag("'")
        ),
        delimited(
            tag("\""),
            alt((
                escaped(none_of("\\\""), '\\', tag("\"")),
                tag("")
            )),
            tag("\"")
        )
    ))(s)
}

fn literal(s: &str) -> IResult<&str, Token> {
    map(
        literal_in,
        |inn| Token::String(inn.to_string())
    )(s)
}

fn boolean(s: &str) -> IResult<&str, Token> {
    map_res(
        alt((tag("true"), tag("false"))),
        |s| bool::from_str(s).and_then(|b| Ok(Token::Boolean(b)))
    )(s)
}

fn symbol(s: &str) -> IResult<&str, Token> {
    map(
        many1(none_of(" ][()\'\"")),
        |ss: Vec<char>| Token::Symbol(ss.into_iter().collect())
    )(s)
}

fn token(s: &str) -> IResult<&str, Token> {
    alt((
        literal, decimal, number, boolean, symbol, list
    ))(s)
}


fn expr(s: &str) -> IResult<&str, Vec<Token>> {
    separated_list(
        many1(one_of(" \t\n")), token
    )(s)
}

fn list(s: &str) -> IResult<&str, Token> {
    map(
        delimited(tag("["), expr, tag("]")),
        |elems| Token::Quotation(Box::new(elems))
    )(s)
}


fn to_quotation(vs: Vec<StackElem>) -> Vec<String> {
    let mut out = Vec::new();
    for v in vs {
        match v {
            StackElem::Number(n) => out.push(n.to_string()),
            StackElem::Boolean(b) => out.push(b.to_string()),
            StackElem::Quotation(q) => out.extend(q.clone()),
            _ => panic!("nope")
        }
    }
    out
}

fn exec_raw(input: &String, mut stack: &mut Vec<StackElem>, mut programs: &mut HashMap<String, Vec<String>>) -> bool {
    let mut vec = input
        .trim().split(" ")
        .map(String::from)
        .collect::<Vec<_>>();
    exec(&mut vec, &mut stack, &mut programs)
}

fn exec(vec: &mut Vec<String>, mut stack: &mut Vec<StackElem>, mut programs: &mut HashMap<String, Vec<String>>) -> bool {
    let mut quit = false;

    let mut in_quotation = false;
    let mut quotation = Vec::new();

    let mut in_definition = false;
    let mut definition = Vec::new();
    let mut definition_name = String::from("");

    vec.reverse();
    while !vec.is_empty() {
        let tok = vec.pop().unwrap();

        if quit {
            return quit;
        }
        if in_quotation {
            if tok == "]" {
                stack.push(StackElem::Quotation(quotation));
                quotation = Vec::new();
                in_quotation = false;
                continue;
            } else {
                quotation.push(tok);
                continue;
            }
        }
        if in_definition {
            if tok == "==" {
                continue;
            }
            definition.push(tok);
            continue;
        }

        match tok.as_ref() {
            "[" => {
                in_quotation = true;
                quotation.clear();
            },
            "+" => {
                let a = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`+` expects two numbers")
                };
                let b = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`+` expects two numbers")
                };
                stack.push(StackElem::Number(a + b));
            },
            "-" => {
                let a = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`+` expects two numbers")
                };
                let b = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`+` expects two numbers")
                };
                stack.push(StackElem::Number(b - a));
            },
            "*" => {
                let a = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`+` expects two numbers")
                };
                let b = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`+` expects two numbers")
                };
                stack.push(StackElem::Number(a * b));
            },
            ">" => {
                let b = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`+` expects two numbers")
                };
                let a = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`+` expects two numbers")
                };
                stack.push(StackElem::Boolean(a > b));
            },
            "<" => {
                let b = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`+` expects two numbers")
                };
                let a = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`+` expects two numbers")
                };
                stack.push(StackElem::Boolean(a < b));
            },
            "=" => {
                let b = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`=` expects two numbers")
                };
                let a = match stack.pop().unwrap() {
                    StackElem::Number(num) => num,
                    _ => panic!("`=` expects two numbers")
                };
                stack.push(StackElem::Boolean(a == b));
            },
            "pop" => {
                stack.pop().unwrap();
            },
            "dup" => {
                let x = stack.pop().unwrap();
                stack.push(x.clone());
                stack.push(x);
            },
            "swap" => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a);
                stack.push(b);
            },
            "concat" => {
                let y = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`concat` expects two quotations")
                };
                let x = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`concat` expects two quotations")
                };
                stack.push(StackElem::Quotation([x, y].concat()));
            },
            "rest" => {
                let x = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`concat` expects two quotations")
                };
                stack.push(StackElem::Quotation(x[1..].to_vec()));
            },
            "size" => {
                let x = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`concat` expects two quotations")
                };
                stack.push(StackElem::Number(x.len() as i32));
            },
            "i" => {
                let mut p = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`concat` expects two quotations")
                };
                p.reverse();
                vec.extend(p);
            },
            "filter" => {
                let filter = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`filter` expects two quotations")
                };
                let original = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`filter` expects two quotations")
                };
                let mut result = vec![];
                for v in original {
                    let mut sub = vec![v.clone()];
                    sub.extend(filter.clone());
                    quit = exec(&mut sub, &mut stack, &mut programs);
                    match stack.pop().unwrap() {
                        StackElem::Boolean(b) => {
                            if b {
                                result.push(v);
                            }
                        },
                        _ => panic!("`filter` predicate must push a boolean")
                    }
                }
                stack.push(StackElem::Quotation(result));
            },
            "fold" => {
                let pred = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`fold` expects a quotation predicate")
                };
                let iv = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`fold` expects a quotation initial value")
                };
                let to_fold = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`fold` expects a quotation to fold")
                };
                let mut sub = iv;
                for v in to_fold {
                    sub.push(v);
                    sub.extend(pred.clone());
                }
                quit = exec(&mut sub, &mut stack, &mut programs);
            },
            "map" => {
                let pred = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`map` expects a quotation predicate")
                };
                let list = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`map` expects a quotation source")
                };
                for v in list {
                    let mut new_stack = vec![];
                    let mut sub = vec![];
                    sub.push(v);
                    sub.extend(pred.clone());
                    quit = exec(&mut sub, &mut new_stack, &mut programs);
                    stack.push(new_stack.pop().unwrap());
                }
            },
            "dip" => {
                let mut prog = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`map` expects a quotation predicate")
                };
                let val = stack.pop().unwrap();
                quit = exec(&mut prog, &mut stack, &mut programs);
                stack.push(val);
            },
            "ifte" => {
                let mut else_pred = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`ifte` expects an else predicate")
                };
                let mut then_pred = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`ifte` expects a then predicate")
                };
                let mut if_pred = match stack.pop().unwrap() {
                    StackElem::Quotation(q) => q,
                    _ => panic!("`ifte` expects an if predicate")
                };
                let mut stack_cloned = stack.clone();
                exec(&mut if_pred, &mut stack_cloned, &mut programs);
                let if_result = match stack_cloned.pop().unwrap() {
                    StackElem::Boolean(b) => b,
                    _ => panic!("`ifte` if predicate must push a boolean")
                };
                if if_result {
                    quit = exec(&mut then_pred, &mut stack, &mut programs);
                } else {
                    quit = exec(&mut else_pred, &mut stack, &mut programs);
                }
            },
            "quit" => {
                quit = true;
                break;
            },
            _ => match tok.parse::<i32>() {
                Ok(num) => stack.push(StackElem::Number(num)),
                Err(_) => {
                    match programs.get(&tok) {
                        Some(p) => {
                            quit = exec(&mut p.clone(), &mut stack, &mut programs);
                        },
                        None => {
                            if tok == definition_name {
                                continue;
                            }
                            println!("defining a program `{}`", tok);
                            definition_name = tok;
                            in_definition = true;
                        }
                    }
                },
            },
        }
    }

    if in_definition {
        programs.insert(definition_name, definition);
    }

    quit
}


fn main() {
    println!("    a joy interpreter");
    let mut stack: Vec<StackElem> = Vec::new();
    let mut quit = false;

    let mut programs = HashMap::new();
    programs.insert(
        "square".to_string(),
        vec!["dup", "*"].iter().map(|x| x.to_string()).collect::<Vec<_>>()
    );

    while !quit {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Err(error) => panic!("error: {}", error),
            _ => {},
        }

        /*quit = exec_raw(&input, &mut stack, &mut programs);
        println!("{:?}", stack);*/
        //println!("{:?}", float32(&input));
        let toks = expr(&input.trim());
        println!("parsed tokens: {:?}", expr(&input.trim()));
    }

}
