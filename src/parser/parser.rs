use crate::error_msg;
use crate::parser::Expr::ArraySuite;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use crate::util::error;

#[derive(Parser)]
#[grammar = "parser_grammar.pest"]
struct ComputeParser;

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub enum Expr {
    Null,
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Box<Vec<Expr>>),
    ArrayParsed(Box<Vec<Vec<Expr>>>),
    ArraySuite(Box<Vec<Expr>>),
    OR(Box<Vec<Expr>>),
    AND(Box<Vec<Expr>>),
    Property(String),
    PropertyFunction(Box<Expr>),
    VariableIdentifier(String),
    FunctionCall(String, Box<Vec<Vec<Expr>>>),
    NamespaceFunctionCall(Vec<String>, String, Box<Vec<Vec<Expr>>>),
    FunctionReturn(Box<Vec<Expr>>),
    Priority(Box<Vec<Expr>>),
    Operation(BasicOperator),
    VariableDeclaration(String, Box<Vec<Expr>>),
    VariableRedeclaration(String, Box<Vec<Expr>>),
    //Condition
    Condition(
        Box<Vec<Expr>>,
        // Code to execute if true
        Box<Vec<Vec<Expr>>>,
        // (OPTIONAL) Code to execute if not true
        Box<Vec<Vec<Expr>>>,
        // (OPTIONAL) Condition of the code to execute if not true
        Box<Vec<Expr>>,
    ),
    // Condition
    While(
        Box<Vec<Expr>>,
        // Code to execute while true
        Box<Vec<Vec<Expr>>>,
    ),
    Loop(String, Box<Expr>, Box<Vec<Vec<Expr>>>),






    // Objects
    File(String)
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq, Copy)]
pub enum BasicOperator {
    Null,
    Add,
    Sub,
    Divide,
    Multiply,
    EQUAL,
    Power,
    AND,
    Modulo,
    Inferior,
    InferiorEqual,
    OR,
    Superior,
    SuperiorEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Variable {
    pub(crate) name: String,
    pub(crate) value: Expr,
}

pub fn parse_expression(pair: Pair<Rule>) -> Vec<Expr> {
    let mut output: Vec<Expr> = vec![];
    let mut recursive = true;

    // println!("{:?}", pair.as_rule());

    match pair.as_rule() {
        Rule::integer => output.push(Expr::Integer(pair.as_str().parse::<i64>().unwrap())),
        Rule::float => output.push(Expr::Float(pair.as_str().parse::<f64>().unwrap())),
        Rule::string => output.push(Expr::String(
            pair.as_str()
                .trim_end_matches("\"")
                .trim_start_matches("\"")
                .parse()
                .unwrap(),
        )),
        Rule::bool => output.push(Expr::Bool({
            if pair.as_str() == "true" {
                true
            } else {
                false
            }
        })),
        // Rule::index_specifier => {
        //     recursive = false;
        //     let input_array = parse_expression(pair.clone().into_inner().next().unwrap())[0].clone();
        //     let index_range = pair.clone().into_inner().skip(1).next().unwrap();
        //     if index_range.as_rule() == Rule::expression {
        //         output.push(Expr::ArrayIndex(Box::from(input_array), Box::from(parse_expression(index_range))))
        //     } else {
        //         todo!()
        //     }
        //
        // }
        Rule::array_suite => {
            recursive = false;
            let mut suite: Vec<Expr> = vec![];
            for extra in pair.clone().into_inner() {
                suite.push(parse_expression(extra)[0].clone());
                // println!("MEM{:?}", suite);
            }

            output.push(ArraySuite(Box::from(suite)))
        }
        Rule::array => {
            // if let Expr::ArrayParsed(x) = &output[output.len()] {
            //
            // }
            let mut array: Vec<Vec<Expr>> = vec![];
            for array_member in pair.clone().into_inner() {
                array.push(parse_expression(array_member))
            }
            recursive = false;
            output.push(Expr::ArrayParsed(Box::from(array)))
        }
        Rule::property => {
            recursive = false;
            output.push(Expr::Property(
                pair.as_str().trim_start_matches(".").parse().unwrap(),
            ))
        }
        Rule::property_function => {
            recursive = false;
            let mut priority_calc: Vec<Vec<Expr>> = vec![];
            for priority_pair in pair
                .clone()
                .into_inner()
                .next()
                .unwrap()
                .into_inner()
                .skip(1)
            {
                for arg_pair in priority_pair.into_inner() {
                    priority_calc.push(parse_expression(arg_pair));
                }
            }
            output.push(Expr::PropertyFunction(Box::from(Expr::FunctionCall(
                pair.clone()
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .trim_start_matches(".")
                    .parse()
                    .unwrap(),
                Box::from(priority_calc),
            ))))
        }
        Rule::func_call => {
            recursive = false;
            let mut priority_calc: Vec<Vec<Expr>> = vec![];
            for priority_pair in pair.clone().into_inner().into_iter().skip(1) {
                for arg_pair in priority_pair.into_inner() {
                    priority_calc.push(parse_expression(arg_pair));
                }
            }
            output.push(Expr::FunctionCall(
                pair.clone()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .parse()
                    .unwrap(),
                Box::from(priority_calc),
            ));
        },
        Rule::func_call_namespace => {
            recursive = false;
            let func_call = parse_expression(pair.clone().into_inner().last().unwrap()).first().unwrap().clone();
            let mut namespaces = vec![];
            for namespace in pair.clone().into_inner().rev().skip(1).rev() {
                namespaces.push(namespace.as_str().to_string());
            }
            println!("{:?}", namespaces);
            if let Expr::FunctionCall(x, y) = func_call {
                output.push(Expr::NamespaceFunctionCall(namespaces, x.clone(), y.clone()));
            } else {
                error(format!("{:?} is not a valid function", func_call).as_str(),"");
            }
        }
        Rule::identifier => if pair.as_str() == "Null" {output.push(Expr::Null)} else {output.push(Expr::VariableIdentifier(
            pair.as_str()
                .trim_end_matches("\"")
                .trim_start_matches("\"")
                .parse()
                .unwrap(),
        ))},
        Rule::priority => {
            recursive = false;
            let mut priority_calc: Vec<Expr> = vec![];
            for priority_pair in pair.clone().into_inner() {
                priority_calc.append(&mut parse_expression(priority_pair));
            }
            output.push(Expr::Priority(Box::from(priority_calc)))
        }
        Rule::ops => match pair.as_str() {
            "+" => output.push(Expr::Operation(BasicOperator::Add)),
            "-" => output.push(Expr::Operation(BasicOperator::Sub)),
            "/" => output.push(Expr::Operation(BasicOperator::Divide)),
            "*" => output.push(Expr::Operation(BasicOperator::Multiply)),
            "==" => output.push(Expr::Operation(BasicOperator::EQUAL)),
            "^" => output.push(Expr::Operation(BasicOperator::Power)),
            "&&" => output.push(Expr::Operation(BasicOperator::AND)),
            "%" => output.push(Expr::Operation(BasicOperator::Modulo)),
            "<" => output.push(Expr::Operation(BasicOperator::Inferior)),
            "<=" => output.push(Expr::Operation(BasicOperator::InferiorEqual)),
            "||" => output.push(Expr::Operation(BasicOperator::OR)),
            ">" => output.push(Expr::Operation(BasicOperator::Superior)),
            ">=" => output.push(Expr::Operation(BasicOperator::SuperiorEqual)),
            _ => todo!(),
        },
        Rule::variableDeclaration => {
            recursive = false;
            let mut priority_calc: Vec<Expr> = vec![];
            for priority_pair in pair.clone().into_inner().into_iter().skip(1) {
                priority_calc.append(&mut parse_expression(priority_pair));
            }
            output.push(Expr::VariableDeclaration(
                pair.clone()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap(),
                Box::from(priority_calc),
            ));
        }
        Rule::variableRedeclaration => {
            recursive = false;
            let mut priority_calc: Vec<Expr> = vec![];
            for priority_pair in pair.clone().into_inner().into_iter().skip(1) {
                priority_calc.append(&mut parse_expression(priority_pair));
            }
            output.push(Expr::VariableRedeclaration(
                pair.clone()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .trim()
                    .parse()
                    .unwrap(),
                Box::from(priority_calc),
            ));
        }
        Rule::and_operation => {
            recursive = false;
            let mut priority_calc: Vec<Expr> = vec![];
            for priority_pair in pair.clone().into_inner().into_iter() {
                priority_calc.append(&mut parse_expression(priority_pair));
            }
            output.push(Expr::AND(Box::from(priority_calc)));
        }
        Rule::or_operation => {
            recursive = false;
            let mut priority_calc: Vec<Expr> = vec![];
            for priority_pair in pair.clone().into_inner().into_iter() {
                priority_calc.append(&mut parse_expression(priority_pair));
            }
            output.push(Expr::OR(Box::from(priority_calc)));
        }
        _ => {}
    }

    if recursive {
        // Recursively process the children
        for inner_pair in pair.into_inner() {
            output.append(&mut parse_expression(inner_pair)); // Increase indent for child nodes
        }
    }
    output
}

fn _visualize_parse_tree(pair: Pair<Rule>, indent: usize) {
    let rule = format!("{:?}", pair.as_rule());
    let span = pair.as_span();
    let text = span.as_str();
    println!("{}{}: \"{}\"", "  ".repeat(indent), rule, text);

    // Recursively process the children
    for inner_pair in pair.into_inner() {
        _visualize_parse_tree(inner_pair, indent + 1);
    }
}

pub fn parse_code(content: &str) -> Vec<Vec<Expr>> {
    let mut instructions: Vec<Vec<Expr>> = vec![];
    for pair in ComputeParser::parse(Rule::code, content)
        .expect(error_msg!("Failed to parse", "Check semicolons and syntax"))
    {
        // _visualize_parse_tree(pair.clone(), 0);
        for inside in pair.into_inner() {
            let mut line_instructions: Vec<Expr> = vec![];
            match inside.as_rule() {
                Rule::expression => {
                    for pair in
                        ComputeParser::parse(Rule::expression, inside.as_str().trim()).unwrap()
                    {
                        line_instructions.append(&mut parse_expression(pair))
                    }
                }
                Rule::if_statement => {
                    let mut condition: Vec<Expr> = vec![];
                    let mut else_code: Vec<Vec<Expr>> = vec![];
                    let mut else_condition: Vec<Expr> = vec![];
                    if let Some(inner_value) =
                        inside.clone().into_inner().into_iter().skip(2).next()
                    {
                        match inner_value.as_rule() {
                            Rule::else_block => {
                                else_code = parse_code(inner_value.into_inner().as_str());
                            }
                            Rule::else_if_block => {
                                for pair in ComputeParser::parse(
                                    Rule::expression,
                                    inside
                                        .clone()
                                        .into_inner()
                                        .into_iter()
                                        .skip(2)
                                        .next()
                                        .unwrap()
                                        .into_inner()
                                        .next()
                                        .unwrap()
                                        .into_inner()
                                        .next()
                                        .unwrap()
                                        .as_str()
                                        .trim(),
                                )
                                .unwrap()
                                {
                                    else_condition.append(&mut parse_expression(pair))
                                }
                                else_code = parse_code(
                                    inner_value
                                        .into_inner()
                                        .next()
                                        .unwrap()
                                        .into_inner()
                                        .skip(1)
                                        .next()
                                        .unwrap()
                                        .as_str(),
                                );
                            }
                            _ => todo!(""),
                        }
                    }
                    for pair in ComputeParser::parse(
                        Rule::expression,
                        inside
                            .clone()
                            .into_inner()
                            .next()
                            .unwrap()
                            .into_inner()
                            .as_str()
                            .trim(),
                    )
                    .unwrap()
                    {
                        condition.append(&mut parse_expression(pair))
                    }
                    line_instructions.push(Expr::Condition(
                        Box::from(condition),
                        Box::from(parse_code(
                            inside
                                .into_inner()
                                .into_iter()
                                .skip(1)
                                .next()
                                .unwrap()
                                .as_str(),
                        )),
                        Box::from(else_code),
                        Box::from(else_condition),
                    ));
                }
                Rule::return_term => line_instructions
                    .push(Expr::FunctionReturn(Box::from(parse_expression(inside)))),
                Rule::while_statement => {
                    let mut condition: Vec<Expr> = vec![];
                    for pair in ComputeParser::parse(
                        Rule::expression,
                        inside
                            .clone()
                            .into_inner()
                            .next()
                            .unwrap()
                            .into_inner()
                            .as_str()
                            .trim(),
                    )
                    .unwrap()
                    {
                        condition.append(&mut parse_expression(pair))
                    }
                    line_instructions.push(Expr::While(
                        Box::from(condition),
                        Box::from(parse_code(
                            inside
                                .into_inner()
                                .into_iter()
                                .skip(1)
                                .next()
                                .unwrap()
                                .as_str(),
                        )),
                    ));
                }
                _ => {}
                Rule::loop_statement => {
                    println!("TEST {:?}", inside.clone().into_inner())
                }
            }
            instructions.push(line_instructions);
        }
    }
    instructions
}