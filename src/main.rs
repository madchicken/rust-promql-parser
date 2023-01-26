// This file is part of PromQL Rust Parser.
// PromQL Rust Parser is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
// PromQL Rust Parser is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
// the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License
// for more details.
// You should have received a copy of the GNU General Public License along with PromQL Rust Parser.
// If not, see <https://www.gnu.org/licenses/>.

mod ast;
mod tokenizer;

extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate core;

use clap::{arg_enum, Parser as Clap_Parser};
use clap::ValueEnum;

use pest::{Parser, state};
use pest::iterators::{Pair, Pairs};
use serde::{Serialize, Deserialize};
use colored::*;
use pest::error::Error;
use serde_json;
use crate::tokenizer::*;


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
                    panic!("Not yet implemented!");
                }
            }

        }
        Err(error) => {
            println!("{:?}", error);
            println!("{}", error.renamed_rules(|r| format!("{:?}", r)))
        }
    }
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