use pest::iterators::Pair;
use serde::{Deserialize, Serialize};
use pest_derive::Parser;
use pest::Parser;

#[derive(Parser)]
#[grammar = "parser_grammar.pest"]
struct ComputeParser;

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
pub enum Expr {
    Null,
    Integer(i64),
    Float(f32),
    String(String),
    Bool(bool),
    VariableIdentifier(String),
    FunctionCall(String, Box<Vec<Vec<Expr>>>),
    FunctionReturn(Box<Vec<Expr>>),
    Priority(Box<Vec<Expr>>),
    Operation(BasicOperator),
    VariableDeclaration(String, Box<Vec<Expr>>),
    Condition(Box<Vec<Expr>>, Box<Vec<Vec<Expr>>>),
}

#[derive(Debug, Serialize, Clone, Deserialize, PartialEq)]
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
    OR,
    Superior
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub(crate) name: String,
    pub(crate) value: Expr
}

pub fn parse_expression(pair: Pair<Rule>) -> Vec<Expr> {
    let mut output: Vec<Expr> = vec![];
    let mut recursive = true;

    // println!("{:?}", pair.as_rule());

    match pair.as_rule() {
        Rule::integer => {
            output.push(Expr::Integer(pair.as_str().parse::<i64>().unwrap()))
        },
        Rule::float => {
            output.push(Expr::Float(pair.as_str().parse::<f32>().unwrap()))
        },
        Rule::string => {
            output.push(Expr::String(pair.as_str().trim_end_matches("\"").trim_start_matches("\"").parse().unwrap()))
        },
        Rule::bool => {
            output.push(Expr::Bool({
                if (pair.as_str() == "true") {
                    true
                } else {
                    false
                }
            }))
        }
        Rule::func_call => {
            recursive = false;
            let mut priority_calc: Vec<Vec<Expr>> = vec![];
            for priority_pair in pair.clone().into_inner().into_iter().skip(1) {
                for arg_pair in priority_pair.into_inner() {
                    priority_calc.push(parse_expression(arg_pair));
                }
            }
            output.push(Expr::FunctionCall(pair.clone().into_inner().next().unwrap().as_str().parse().unwrap(), Box::from(priority_calc)));

        }
        Rule::identifier => {
            output.push(Expr::VariableIdentifier(pair.as_str().trim_end_matches("\"").trim_start_matches("\"").parse().unwrap()))
        }
        Rule::priority => {
            recursive = false;
            let mut priority_calc: Vec<Expr> = vec![];
            for priority_pair in pair.clone().into_inner() {
                priority_calc.append(&mut parse_expression(priority_pair));
            }
            output.push(Expr::Priority(Box::from(priority_calc)))
        },
        Rule::ops => {
            match pair.as_str() {
                "+" => {
                    output.push(Expr::Operation(BasicOperator::Add))
                },
                "-" => {
                    output.push(Expr::Operation(BasicOperator::Sub))
                },
                "/" => {
                    output.push(Expr::Operation(BasicOperator::Divide))
                },
                "*" => {
                    output.push(Expr::Operation(BasicOperator::Multiply))
                },
                "==" => {
                    output.push(Expr::Operation(BasicOperator::EQUAL))
                },
                "^" => {
                    output.push(Expr::Operation(BasicOperator::Power))
                },
                "&&" => {
                    output.push(Expr::Operation(BasicOperator::AND))
                },
                "%" => {
                    output.push(Expr::Operation(BasicOperator::Modulo))
                },
                "<" => {
                    output.push(Expr::Operation(BasicOperator::Inferior))
                },
                "||" => {
                    output.push(Expr::Operation(BasicOperator::OR))
                },
                ">" => {
                    output.push(Expr::Operation(BasicOperator::Superior))
                }
                _ => todo!()
            }
        },
        Rule::variableDeclaration => {
            recursive = false;
            // println!("{:?}", pair.clone().into_inner().next().unwrap().as_str());
            let mut priority_calc: Vec<Expr> = vec![];
            for priority_pair in pair.clone().into_inner().into_iter().skip(1) {
                priority_calc.append(&mut parse_expression(priority_pair));
            }
            output.push(Expr::VariableDeclaration(pair.clone().into_inner().next().unwrap().as_str().trim().parse().unwrap(), Box::from(priority_calc)));
        }
        _ => {}
    }

    if recursive {
        // Recursively process the children
        for inner_pair in pair.into_inner() {
            output.append(&mut parse_expression(inner_pair));  // Increase indent for child nodes
        }
    }
    output
}

fn _visualize_parse_tree(pair: Pair<Rule>, indent: usize) {
    let rule = format!("{:?}", pair.as_rule());
    let span = pair.as_span();
    let text = span.as_str();
    println!(
        "{}{}: \"{}\"",
        "  ".repeat(indent),
        rule,
        text
    );

    // Recursively process the children
    for inner_pair in pair.into_inner() {
        _visualize_parse_tree(inner_pair, indent + 1);
    }
}

pub fn parse_code(content: &str) -> Vec<Vec<Expr>> {
    let mut instructions: Vec<Vec<Expr>> = vec![];
    for pair in ComputeParser::parse(Rule::code, content).unwrap() {
        // _visualize_parse_tree(pair.clone(), 0);
        for inside in pair.into_inner() {
            let mut line_instructions: Vec<Expr> = vec![];
            match inside.as_rule() {
                Rule::expression => {
                    for pair in ComputeParser::parse(Rule::expression, inside.as_str().trim()).unwrap() {
                        line_instructions.append(&mut parse_expression(pair))
                    }
                },
                Rule::if_statement => {
                    let mut condition: Vec<Expr> = vec![];
                    for pair in ComputeParser::parse(Rule::expression, inside.clone().into_inner().next().unwrap().into_inner().as_str().trim()).unwrap() {
                        condition.append(&mut parse_expression(pair))
                    }
                    line_instructions.push(Expr::Condition(Box::from(condition), Box::from(parse_code(inside.into_inner().into_iter().skip(1).next().unwrap().as_str()))));
                },
                Rule::return_term => {
                    // println!("is thtfe{:?}", parse_expression(inside));
                    line_instructions.push(Expr::FunctionReturn(Box::from(parse_expression(inside))))
                },
                _ => {

                }
            }
            instructions.push(line_instructions);
        }
    }
    instructions
}