mod parser;
mod parser_functions;
mod util;

use crate::parser::{BasicOperator, Expr, Variable};
use crate::parser_functions::parse_functions;
use crate::util::{assert_args_number, check_and_or, error};
use inflector::Inflector;
use std::fs;
use std::ops::Index;

fn process_stack(
    mut stack: Vec<Expr>,
    variables: Vec<Variable>,
    functions: Vec<(&str, Vec<&str>, Vec<Vec<Expr>>)>,
) -> Expr {
    let mut output: Expr = Expr::Null;
    let mut current_operator: BasicOperator = BasicOperator::Null;
    let mut and_or_operator: BasicOperator = BasicOperator::Null;
    // println!("{:?}", stack);
    for x in &mut stack {
        if let Expr::VariableIdentifier(ref var) = x {
            let variable = variables
                .iter()
                .filter(|variable| variable.name == *var)
                .next()
                .unwrap();
            *x = variable.value.clone();
        } else if let Expr::FunctionCall(ref func_name, ref func_args) = x {
            // replace function call by its result (return value)
            let args: Vec<Expr> = func_args
                .iter()
                .map(|arg| process_stack(arg.clone(), variables.clone(), functions.clone()))
                .collect();
            let target_function: (&str, Vec<&str>, Vec<Vec<Expr>>) = functions
                .clone()
                .into_iter()
                .filter(|func| func.0 == func_name)
                .next()
                .expect(&format!("Unknown function '{}'", func_name));
            assert_args_number(&func_name, args.len(), target_function.1.len());
            let target_args: Vec<Variable> = target_function
                .1
                .iter()
                .enumerate()
                .map(|(i, arg)| Variable {
                    name: arg.parse().unwrap(),
                    value: args[i].clone(),
                })
                .collect();
            let result = process_function(
                target_function.2,
                target_args,
                target_function.1,
                target_function.0,
                functions.clone(),
            );
            *x = result;
        }
    }

    // stack = stack.into_iter().map(|stackitem|
    //     if let Expr::Priority(x) = stackitem {
    //         return
    //     } else {
    //         return stackitem;
    //     }
    // ).collect();

    for element in stack {
        if output == Expr::Null {
            output = element
        } else {
            match element {
                Expr::Operation(BasicOperator::AND) => {
                    and_or_operator = BasicOperator::AND;
                    println!("AND")
                }
                Expr::Operation(BasicOperator::OR) => {
                    and_or_operator = BasicOperator::OR;
                    println!("OR");
                }
                Expr::Operation(op) => {
                    current_operator = op;
                }
                Expr::String(x) => {
                    if matches!(output, Expr::String(ref value)) {
                        match current_operator {
                            BasicOperator::Add => {
                                if let Expr::String(value) = output {
                                    output = Expr::String(value + &x);
                                }
                            }
                            _ => todo!(),
                        }
                    } else {
                        error("oh", "");
                    }
                }
                Expr::Float(x) => {
                    if matches!(output, Expr::Integer(value)) {
                        match current_operator {
                            BasicOperator::Add => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Float(value as f32 + x);
                                }
                            }
                            BasicOperator::Sub => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Float(value as f32 - x);
                                }
                            }
                            BasicOperator::Divide => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Float(value as f32 / x);
                                }
                            }
                            BasicOperator::Multiply => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Float(value as f32 * x);
                                }
                            }
                            BasicOperator::Power => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Float((value as f32).powf(x));
                                }
                            }
                            _ => todo!(),
                        }
                    }
                }
                Expr::Integer(x) => {
                    if matches!(output, Expr::Integer(_)) {
                        match current_operator {
                            BasicOperator::Add => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Integer(value + x);
                                }
                            }
                            BasicOperator::Sub => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Integer(value - x);
                                }
                            }
                            BasicOperator::Divide => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Float(value as f32 / x as f32);
                                }
                            }
                            BasicOperator::Multiply => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Integer(value * x);
                                }
                            }
                            BasicOperator::Power => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Integer(value.pow(x as u32));
                                }
                            }
                            BasicOperator::EQUAL => {
                                if let Expr::Integer(value) = output {
                                    output = check_and_or(
                                        and_or_operator,
                                        output,
                                        Expr::Bool(value == x),
                                    );
                                    and_or_operator = BasicOperator::Null;
                                }
                            }
                            BasicOperator::Inferior => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Bool(value < x);
                                }
                            }
                            BasicOperator::InferiorEqual => {
                                if let Expr::Integer(value) = output {
                                    output = check_and_or(
                                        and_or_operator,
                                        output,
                                        Expr::Bool(value <= x),
                                    );
                                    and_or_operator = BasicOperator::Null;
                                }
                            }
                            BasicOperator::Superior => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Bool(value > x);
                                }
                            }
                            BasicOperator::SuperiorEqual => {
                                if let Expr::Integer(value) = output {
                                    output = Expr::Bool(value >= x);
                                }
                            }
                            _ => todo!(""),
                        }
                    }
                }
                Expr::Property(x) => {
                    if matches!(output, Expr::String(_)) {
                        match x.as_str() {
                            "" => {}
                            _ => {}
                        }
                    }
                }
                Expr::PropertyFunction(z) => {
                    if let Expr::FunctionCall(x, y) = *z {
                        let args: Vec<Expr> = y
                            .iter()
                            .map(|arg| {
                                process_stack(arg.clone(), variables.clone(), functions.clone())
                            })
                            .collect();

                        // STR
                        if let Expr::String(str) = output.clone() {
                            match x.as_str() {
                                // TRANSFORM
                                "uppercase" => {
                                    assert_args_number("uppercase", args.len(), 0);
                                    output = Expr::String(str.to_uppercase());
                                }
                                "lowercase" => {
                                    assert_args_number("lowercase", args.len(), 0);
                                    output = Expr::String(str.to_lowercase());
                                }
                                "capitalize" => {
                                    assert_args_number("capitalize", args.len(), 0);
                                    output = Expr::String(str.to_title_case());
                                }
                                "replace" => {
                                    assert_args_number("replace", args.len(), 2);
                                    if let Expr::String(toreplace) = &args[0] {
                                        if let Expr::String(replaced) = &args[1] {
                                            output = Expr::String(str.replace(toreplace, replaced))
                                        } else {
                                            error(
                                                format!("{:?} is not a String", &args[1]).as_str(),
                                                format!("Convert {:?} to a String", &args[1])
                                                    .as_str(),
                                            );
                                        }
                                    } else {
                                        error(
                                            format!("{:?} is not a String", &args[0]).as_str(),
                                            format!("Convert {:?} to a String", &args[0]).as_str(),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => todo!(),
            }
        }
    }
    output
}

fn process_function(
    lines: Vec<Vec<Expr>>,
    included_variables: Vec<Variable>,
    expected_variables: Vec<&str>,
    name: &str,
    functions: Vec<(&str, Vec<&str>, Vec<Vec<Expr>>)>,
) -> Expr {
    if (included_variables.len() != expected_variables.len()) {
        error(
            &format!(
                "Function '{}' expected {} arguments, but received {}",
                name,
                expected_variables.len(),
                included_variables.len()
            ),
            "Remove the excess arguments",
        )
    }
    let mut variables: Vec<Variable> = included_variables;
    for instructions in lines {
        for instruction in instructions {
            // println!("{:?}", instruction);
            match instruction {
                Expr::VariableDeclaration(x, y) => variables.push(Variable {
                    name: x,
                    value: process_stack(*y, variables.clone(), functions.clone()),
                }),
                Expr::FunctionCall(x, y) => {
                    // println!("{:?}", y);
                    let args: Vec<Expr> = y
                        .iter()
                        .map(|arg| process_stack(arg.clone(), variables.clone(), functions.clone()))
                        .collect();
                    if x == "print" {
                        assert_args_number("print", args.len(), 1);
                        match &args[0] {
                            Expr::Float(val) => {
                                println!("{}", val)
                            }
                            Expr::Integer(val) => {
                                println!("{}", val)
                            }
                            Expr::String(val) => {
                                println!("{}", val);
                            }
                            Expr::Bool(val) => {
                                println!("{}", val)
                            }
                            _ => error(&format!("Cannot print {:?} type", &args[0]), "Change type"),
                        }
                    } else {
                        let target_function: (&str, Vec<&str>, Vec<Vec<Expr>>) = functions
                            .clone()
                            .into_iter()
                            .filter(|func| func.0 == x)
                            .next()
                            .expect(&format!("Unknown function '{}'", x));
                        assert_args_number(&x, args.len(), target_function.1.len());
                        let target_args: Vec<Variable> = target_function
                            .1
                            .iter()
                            .enumerate()
                            .map(|(i, arg)| Variable {
                                name: arg.parse().unwrap(),
                                value: args[i].clone(),
                            })
                            .collect();
                        process_function(
                            target_function.2,
                            target_args,
                            target_function.1,
                            target_function.0,
                            functions.clone(),
                        );
                        // println!("{:?}", target_args)
                    }
                }
                Expr::FunctionReturn(x) => {
                    return process_stack(*x, variables.clone(), functions.clone())
                }
                Expr::Condition(x, y) => {
                    let condition = process_stack(*x, variables.clone(), functions.clone());
                    if (condition == Expr::Bool(true)) {
                        process_function(
                            *y,
                            variables.clone(),
                            variables
                                .iter()
                                .map(|variable| variable.name.as_str())
                                .collect(),
                            name,
                            functions.clone(),
                        );
                    }
                }
                _ => todo!(),
            }
            // println!("{:?}", instruction)
        }
        // println!("{:?}", instructions)
    }
    Expr::Null
    // println!("{:?}", variables)
}

fn main() {
    let filename = "example.compute";

    let content = fs::read_to_string(filename).unwrap();

    let functions: Vec<(&str, Vec<&str>, Vec<Vec<Expr>>)> =
        parse_functions(&content, filename.parse().unwrap());
    println!("{:?}", functions);

    let main_instructions = functions
        .clone()
        .into_iter()
        .filter(|function| function.0 == "main")
        .collect::<Vec<(&str, Vec<&str>, Vec<Vec<Expr>>)>>()
        .first()
        .unwrap()
        .clone()
        .2;
    process_function(main_instructions, vec![], vec![], "main", functions);

    // println!("{:?}", process_stack(vec![Expr::Integer(32), Expr::Operation(BasicOperator::Add), Expr::Float(5.6)]))
}
