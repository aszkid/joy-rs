use std::io::{self, Write};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum StackElem {
    Number(i32),
    Boolean(bool),
    Quotation(Vec<String>),
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
                quit = exec(&mut if_pred, &mut stack_cloned, &mut programs);
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

        quit = exec_raw(&input, &mut stack, &mut programs);
        println!("{:?}", stack);
    }

}
