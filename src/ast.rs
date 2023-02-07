// This file is part of PromQL Rust Parser.
// PromQL Rust Parser is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
// PromQL Rust Parser is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
// the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License
// for more details.
// You should have received a copy of the GNU General Public License along with PromQL Rust Parser.
// If not, see <https://www.gnu.org/licenses/>.

use strum_macros::EnumString;

#[derive(EnumString)]
#[derive(Debug)]
enum Unit {
    #[strum(serialize="ms")]
    MS,
    #[strum(serialize="s")]
    S,
    #[strum(serialize="m")]
    M,
    #[strum(serialize="h")]
    H,
    #[strum(serialize="d")]
    D,
    #[strum(serialize="w")]
    W,
    #[strum(serialize="y")]
    Y,
}

#[derive(EnumString)]
#[derive(Debug)]
pub enum Op {
    #[strum(serialize="+")]
    Add,
    #[strum(serialize="-")]
    Subtract,
    #[strum(serialize="*")]
    Multiply,
    #[strum(serialize="/")]
    Divide,
    #[strum(serialize="^")]
    Pow,
    #[strum(serialize="%")]
    Modulo,
    #[strum(serialize=">")]
    Gt,
    #[strum(serialize=">=")]
    Gte,
    #[strum(serialize="<")]
    Lt,
    #[strum(serialize="<=")]
    Lte,
    #[strum(serialize="==")]
    Eq,
    #[strum(serialize="!=")]
    Ne,
    #[strum(serialize="and")]
    And,
    #[strum(serialize="or")]
    Or,
    #[strum(serialize="unless")]
    Unless
}

#[derive(Debug, Clone)]
pub struct LabelAndValue {
    pub name: String,
    pub op: String,
    pub value: String
}

#[derive(Debug)]
pub struct Scope {
    pub labels: Vec<LabelAndValue>
}

#[derive(Debug)]
pub struct Param {
    pub val_float: Option<f32>,
    pub val_string: Option<String>
}

#[derive(Debug)]
pub struct Metric {
    pub metric: String,
    pub scope: Option<Scope>
}

#[derive(Debug)]
pub enum InstantVector {
    Scalar(f32),
    Func(Box<Function>),
    MetricWithScope(Metric),
    NameScope(Scope)
}

#[derive(Debug)]
pub struct RangeVector {
    instant_vector: InstantVector,
    range: u64,
    range_unit: Unit
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Option<Vec<Param>>,
    pub instant_vector: Option<InstantVector>,
    pub range_vector: Option<RangeVector>
}

#[derive(Debug)]
enum Sign {
    Plus, Minus
}

#[derive(Debug)]
enum Match {
    On, Ignoring
}

#[derive(Debug)]
struct VectorMatching {
    matching: Match,
    labels: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum Expr {
    Sign(Box<Expr>),
    InstantVector(InstantVector),
    Bool(Box<Expr>),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    }
}

