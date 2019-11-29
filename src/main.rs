use std::io::{self, Write};

#[derive(Debug, Clone)]
enum StackElem {
    Number(i32),
    Quotation(Vec<String>),
}

fn parse_to_stack(input: &String, stack: &mut Vec<StackElem>) -> bool {
    let vec = input.trim().split(" ").collect::<Vec<_>>();
    let mut quit = false;
    let mut program = Vec::new();
    let mut in_quotation = false;

    for tok in vec {
        if in_quotation {
            if tok == "]" {
                stack.push(StackElem::Quotation(program.clone()));
                in_quotation = false;
                println!("closing quotation...");
                continue;
            } else {
                println!("pushing to quotation...");
                program.push(String::from(tok));
                continue;
            }
        }

        match tok.as_ref() {
            "[" => {
                in_quotation = true;
                program.clear();
                println!("opening quotation...");
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
