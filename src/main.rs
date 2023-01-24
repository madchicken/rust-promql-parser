mod ast;
extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate core;

use clap::{arg_enum, Parser as Clap_Parser};
use clap::ValueEnum;

use pest::Parser;
use pest::iterators::Pair;
use serde::{Serialize, Deserialize};
use colored::*;
use serde_json;

#[derive(Parser)]
#[grammar = "promql.pest"]
struct PromQLParser;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum SyntaxTokenType {
    Metric,
    Label,
    Value,
    Bracket,
    Scalar,
    Keyword,
    Function,
    Text
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SyntaxTokenPosition {
    offset: usize,
    line: usize,
    col: usize
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SyntaxToken {
    start: SyntaxTokenPosition,
    end: SyntaxTokenPosition,
    token_type: SyntaxTokenType,
    content: String
}

fn token_from_rule(rule: Rule) -> SyntaxTokenType {
    match rule {
        Rule::metric => SyntaxTokenType::Metric,
        Rule::label => SyntaxTokenType::Label,
        Rule::scalar => SyntaxTokenType::Scalar,
        Rule::sq_inner => SyntaxTokenType::Value,
        Rule::dq_inner => SyntaxTokenType::Value,
        Rule::bq_inner => SyntaxTokenType::Value,
        Rule::string_literal => SyntaxTokenType::Value,
        Rule::function => SyntaxTokenType::Function,
        Rule::aggregation => SyntaxTokenType::Function,
        Rule::lparen => SyntaxTokenType::Bracket,
        Rule::rparen => SyntaxTokenType::Bracket,
        Rule::lsquare => SyntaxTokenType::Bracket,
        Rule::rsquare => SyntaxTokenType::Bracket,
        Rule::lcurly => SyntaxTokenType::Bracket,
        Rule::rcurly => SyntaxTokenType::Bracket,
        _ => SyntaxTokenType::Text
    }
}

fn to_token_pos(pos: pest::Position) -> SyntaxTokenPosition {
    let ( line, col) = pos.line_col();
    let offset = pos.pos();
    SyntaxTokenPosition {
        offset, line, col
    }
}

fn get_tokens(pair: Pair<Rule>) -> Vec<SyntaxToken> {
    let children: Vec<_> = pair.clone().into_inner().collect();
    let len = children.len();
    if len > 0 {
        return children.into_iter().map(|child | get_tokens(child)).flatten().collect();
    }

    vec![SyntaxToken {
        start: to_token_pos(pair.as_span().start_pos()),
        end: to_token_pos(pair.as_span().end_pos()),
        token_type: token_from_rule(pair.as_rule()),
        content: String::from(pair.as_span().as_str())
    }]
}

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
        SyntaxTokenType::Label=> token.content.cyan().to_string(),
        SyntaxTokenType::Metric => token.content.italic().red().to_string(),
        SyntaxTokenType::Value=> token.content.green().to_string(),
        SyntaxTokenType::Bracket=> token.content.blue().to_string(),
        SyntaxTokenType::Scalar=> token.content.yellow().to_string(),
        SyntaxTokenType::Keyword=> format!("{} ", token.content.red()),
        SyntaxTokenType::Function=> format!("{} ", token.content.purple()),
        SyntaxTokenType::Text=> token.content.blue().to_string(),
    }
}

fn parse_promql(query: &str, output: Option<Output>) {
    let prom_ql = PromQLParser::parse(Rule::query, query);
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
            }

        }
        Err(error) => println!("{}", error.renamed_rules(|r| format!("{:?}", r)))
    }
}

arg_enum! {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
    enum Output {
        Json,
        Tree
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