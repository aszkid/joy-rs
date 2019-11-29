use std::io::{self, Write};

#[derive(Debug, Clone)]
enum StackElem {
    Number(i32),
    Boolean(bool),
    Quotation(Vec<String>),
}

fn exec_raw(input: &String, mut stack: &mut Vec<StackElem>) -> bool {
    let mut vec = input
        .trim().split(" ")
        .map(String::from)
        .collect::<Vec<_>>();
    exec(&mut vec, &mut stack)
}

fn exec(vec: &mut Vec<String>, mut stack: &mut Vec<StackElem>) -> bool {
    let mut quit = false;
    let mut program = Vec::new();
    let mut in_quotation = false;
    vec.reverse();

    while !vec.is_empty() {
        let tok = vec.pop().unwrap();

        if in_quotation {
            if tok == "]" {
                stack.push(StackElem::Quotation(program));
                program = Vec::new();
                in_quotation = false;
                continue;
            } else {
                program.push(tok);
                continue;
            }
        }

        match tok.as_ref() {
            "[" => {
                in_quotation = true;
                program.clear();
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
                    quit = exec(&mut sub, &mut stack);
                    match stack.pop().unwrap() {
                        StackElem::Boolean(b) => {
                            if b {
                                result.push(v);
                            }
                        },
                        _ => panic!("`filter` predicate must return a boolean")
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
                quit = exec(&mut sub, &mut stack);
            },
            "quit" => {
                quit = true;
                break;
            },
            _ => match tok.parse::<i32>() {
                Ok(num) => stack.push(StackElem::Number(num)),
                Err(_) => println!("unknown token: {}", tok),
            },
        }
    }

    quit
}


fn main() {
    println!("    a joy interpreter");
    let mut stack: Vec<StackElem> = Vec::new();
    let mut quit = false;

    while !quit {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Err(error) => panic!("error: {}", error),
            _ => {},
        }

        quit = exec_raw(&input, &mut stack);
        println!("{:?}", stack);
    }

}
