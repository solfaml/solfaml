pub mod ast;
pub mod parser;

#[cfg(feature = "wasm")]
pub mod wasm;

use winnow::{
    Parser,
    error::{ContextError, ParseError},
};

use crate::{ast::Solfa, parser::solfa_parser};

pub fn parse_solfa(source: &str) -> Result<Solfa, ParseError<&str, ContextError>> {
    solfa_parser.parse(source)
}
