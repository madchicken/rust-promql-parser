// This file is part of PromQL Rust Parser.
// PromQL Rust Parser is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, either version 3 of the License,
// or (at your option) any later version.
// PromQL Rust Parser is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even
// the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License
// for more details.
// You should have received a copy of the GNU General Public License along with PromQL Rust Parser.
// If not, see <https://www.gnu.org/licenses/>.

use std::iter::Map;
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
enum Op {
    #[strum(serialize="+")]
    Plus,
    #[strum(serialize="-")]
    Minus,
    #[strum(serialize="*")]
    Multiply,
    #[strum(serialize="/")]
    Division,
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

#[derive(Debug)]
struct Scope {
    labels: Map<String, String>
}

#[derive(Debug)]
struct Param {
    val_int: Option<u32>,
    val_float: Option<f32>,
    val_string: Option<String>
}

#[derive(Debug)]
struct InstantVector {
    function: Option<Box<Function>>,
    metric: String,
    scope: Option<Box<Scope>>,
}

#[derive(Debug)]
struct RangeVector {
    instant_vector: InstantVector,
    range: u64,
    range_unit: Unit
}

#[derive(Debug)]
struct Function {
    name: String,
    params: Option<Vec<Param>>,
    instant_vector: Option<InstantVector>,
    range_vector: Option<RangeVector>
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
    Sign(Option<Sign>),
    Scalar(Option<f32>),
    InstantVector(Option<InstantVector>),
    BinOp {
        lhs: Box<Expr>,
        op: Option<Op>,
        rhs: Option<Box<Expr>>,
    }
}

