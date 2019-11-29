use std::io::{self, Write};

#[derive(Debug, Clone)]
enum StackElem {
    Number(i32),
    Quotation(Vec<String>),
}

fn parse_to_stack(input: &String, stack: &mut Vec<StackElem>) -> bool {
    let mut vec = input.trim().split(" ").collect::<Vec<_>>();
    vec.reverse();
    let mut quit = false;
    let mut program = Vec::new();
    let mut buffer: Vec<String> = Vec::new();
    let mut in_quotation = false;

    while !vec.is_empty() || !buffer.is_empty() {
        let mut tok = if !buffer.is_empty() {
            buffer.pop().unwrap()
        } else {
            String::from(vec.pop().unwrap())
        };

        if in_quotation {
            if tok == "]" {
                stack.push(StackElem::Quotation(program.clone()));
                in_quotation = false;
                continue;
            } else {
                program.push(String::from(tok));
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
            "dup" => {
                let x = stack.pop().unwrap();
                stack.push(x.clone());
                stack.push(x.clone());
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
                buffer = p;
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

        quit = parse_to_stack(&input, &mut stack);
        println!("{:?}", stack);
    }

}
