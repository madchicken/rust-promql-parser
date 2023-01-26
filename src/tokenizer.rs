// This file is part of PromQL Rust Parser.
// PromQL Rust Parser is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
// PromQL Rust Parser is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
// the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License
// for more details.
// You should have received a copy of the GNU General Public License along with PromQL Rust Parser.
// If not, see <https://www.gnu.org/licenses/>.

use pest::Parser;
use pest::iterators::Pair;
use serde::{Serialize, Deserialize};

#[derive(Parser)]
#[grammar = "promql.pest"]
pub struct PromQLParser;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SyntaxTokenType {
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
pub struct SyntaxTokenPosition {
    pub offset: usize,
    pub line: usize,
    pub col: usize
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyntaxToken {
    pub start: SyntaxTokenPosition,
    pub end: SyntaxTokenPosition,
    pub token_type: SyntaxTokenType,
    pub content: String
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

pub fn get_tokens(pair: Pair<Rule>) -> Vec<SyntaxToken> {
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
