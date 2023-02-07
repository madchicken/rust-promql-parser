// This file is part of PromQL Rust Parser.
// PromQL Rust Parser is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
// PromQL Rust Parser is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
// the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License
// for more details.
// You should have received a copy of the GNU General Public License along with PromQL Rust Parser.
// If not, see <https://www.gnu.org/licenses/>.

pub mod ast;
mod tokenizer;

extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate core;


use clap::{arg_enum, Parser as Clap_Parser};
use clap::ValueEnum;

use pest::{Parser};
use pest::iterators::{Pair, Pairs};

use colored::*;
use pest::error::Error;
use serde_json;
use crate::tokenizer::*;
use pest::pratt_parser::{PrattParser};
use crate::ast::{Function, Param, Op, Metric, Scope, Expr, InstantVector, LabelAndValue};
use crate::ast::InstantVector::{Func, MetricWithScope, NameScope};
use crate::Expr::Bool;


fn format_pair(pair: Pair<Rule>, indent_level: usize, is_newline: bool) -> String {
    let indent = if is_newline {
        "  ".repeat(indent_level)
    } else {
        "".to_string()
    };

    let children: Vec<_> = pair.clone().into_inner().collect();
    let len = children.len();
    let children: Vec<_> = children.into_iter().map(|pair| {
        format_pair(pair, if len > 1 { indent_level + 1 } else { indent_level }, len > 1)
    }).collect();

    let dash = if is_newline {
        "- "
    } else {
        ""
    };

    match len {
        0 => format!("{}{}{:?}: {:?}", indent, dash, pair.as_rule(), pair.as_span().as_str()),
        1 => format!("{}{}{:?} > {}", indent, dash, pair.as_rule(), children[0]),
        _ => format!("{}{}{:?}\n{}", indent, dash, pair.as_rule(), children.join("\n"))
    }
}

fn format_token(token: SyntaxToken) -> String {
    match token.token_type {
        SyntaxTokenType::Label => token.content.cyan().to_string(),
        SyntaxTokenType::Metric => token.content.italic().red().to_string(),
        SyntaxTokenType::Value => token.content.green().to_string(),
        SyntaxTokenType::Bracket => token.content.blue().to_string(),
        SyntaxTokenType::Scalar => token.content.yellow().to_string(),
        SyntaxTokenType::Keyword => format!("{} ", token.content.red()),
        SyntaxTokenType::Function => format!("{} ", token.content.purple()),
        SyntaxTokenType::Text => token.content.blue().to_string(),
    }
}

fn parse_promql(query: &str, output: Option<Output>) {
    let prom_ql: Result<Pairs<Rule>, Error<Rule>> = PromQLParser::parse(Rule::query, query);
    match prom_ql {
        Ok(pairs) => {
            let pc = pairs.clone();
            let x: Vec<SyntaxToken> = pairs
                .map(|pair| get_tokens(pair))
                .flatten()
                .collect();

            let output = output.unwrap_or(Output::Tree);
            match output {
                Output::Json => {
                    println!("{}", serde_json::to_string_pretty(&x.clone()).unwrap());
                },
                Output::Tree => {
                    let lines: Vec<String> = pc.map(|pair| format_pair(pair, 0, true)).collect();
                    println!("{}", lines.join("\n"));
                },
                Output::Ast => {
                    let expr = parse_expr(pc.peek().unwrap().into_inner());
                    println!("{:?}", expr);
                }
            }

        }
        Err(error) => {
            println!("{:?}", error);
            println!("{}", error.renamed_rules(|r| format!("{:?}", r)))
        }
    }
}

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            // Addition and subtract have equal precedence
            .op(Op::infix(add, Left) | Op::infix(subtract, Left))
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left) | Op::infix(modulo, Left))
            .op(Op::prefix(unary_sign))
    };
}

fn parse_param(pairs: Pairs<Rule>) -> Param {
    let mut param = Param {
        val_float: None,
        val_string: None
    };
    match pairs.peek().unwrap().as_rule() {
        Rule::scalar => param.val_float = Some(pairs.as_str().parse::<f32>().unwrap()),
        Rule::string_literal => param.val_string = Some(String::from(pairs.as_str())),
        _ => panic!("Can't parse param {:?}", pairs.as_str())
    };
    param
}

// fn parse_function(pairs: Pairs<Rule>) -> Function {
//
// }

fn parse_scope(pairs: Pairs<Rule>) -> Option<Scope> {
    let labels: Vec<LabelAndValue> = pairs
        .flatten()
        .filter(|p| p.as_rule() == Rule::pair || p.as_rule() == Rule::name_pair)
        .map(|p| {
            let rules: Vec<Pair<Rule>> = p.into_inner().flatten().collect();
            let name = String::from(rules.get(0)?.as_str());
            let op = String::from(rules.get(1)?.as_str());
            let value = String::from(rules.get(2)?.as_str());
            Some(LabelAndValue {
                name,
                op,
                value
            })
        })
        .filter(|o| o.is_some())
        .map(|o| o.unwrap())
        .collect();
    Some(Scope {
        labels
    })
}

fn parse_name_scope(mut pairs: Pairs<Rule>) -> Scope {
    parse_scope(pairs).unwrap()
}

fn parse_metric_with_scope(mut pairs: Pairs<Rule>) -> Metric {
    let metric = pairs.next().unwrap();
    Metric {
        metric: String::from(metric.as_str()),
        scope: if pairs.peek().is_some() { parse_scope(pairs.peek().unwrap().into_inner()) } else { None }
    }
}

fn parse_instant_vector(pairs: Pairs<Rule>) -> InstantVector {
    let pair = pairs.peek().unwrap();
    match pair.as_rule() {
        // Rule::functions_to_instant => Func(Box::new(parse_function(pair.into_inner()))),
        Rule::metric_with_scope => MetricWithScope(parse_metric_with_scope(pair.into_inner())),
        Rule::name_vector => NameScope(parse_name_scope(pair.into_inner())),
        _ => panic!("Can't parse instant vector {:?}", pair.as_str())
    }
}

fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    let primary = pairs.peek().unwrap();
    match primary.as_rule() {
        Rule::expr => parse_expr(primary.into_inner()),
        Rule::instant_vector => Expr::InstantVector(parse_instant_vector(primary.into_inner())),
        _ => unreachable!("Expr::parse expected expr, found {:?}", primary),
    }
    // PRATT_PARSER
    //     .map_primary(|primary| match primary.as_rule() {
    //         Rule::expr => parse_expr(primary.into_inner()),
    //         Rule::instant_vector => Expr::InstantVector(parse_instant_vector(primary.into_inner())),
    //         Rule::postfix => Bool(Box::new(parse_expr(primary.into_inner()))),
    //         rule => unreachable!("Expr::parse expected atom, found {:?}", primary),
    //     })
    //     .map_infix(|lhs, op, rhs| {
    //         let op = match op.as_rule() {
    //             Rule::add => Op::Add,
    //             Rule::subtract => Op::Subtract,
    //             Rule::multiply => Op::Multiply,
    //             Rule::divide => Op::Divide,
    //             Rule::modulo => Op::Modulo,
    //             rule => unreachable!("Expr::parse expected infix operation, found {:?}", op),
    //         };
    //         Expr::BinOp {
    //             lhs: Box::new(lhs),
    //             op,
    //             rhs: Box::new(rhs),
    //         }
    //     })
    //     .map_prefix(|op, rhs| match op.as_rule() {
    //         Rule::unary_sign => Expr::Sign(Box::new(rhs)),
    //         _ => unreachable!(),
    //     })
    //     .parse(pairs)
}

arg_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
    enum Output {
        Json,
        Tree,
        Ast
    }
}

#[derive(Clap_Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Params {
    /// Query to operate on
    query: String,

    output: Option<Output>
}

fn main() {
    let cli = Params::parse();
    parse_promql(cli.query.as_str(), cli.output);
}