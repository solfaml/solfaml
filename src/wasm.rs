use wasm_bindgen::{JsError, prelude::*};
use winnow::Parser;

use crate::{ast::Solfa, parser::solfa_parser};

#[wasm_bindgen(js_name = "parseSolfa")]
pub fn parse_solfa(source: &str) -> Result<Solfa, JsError> {
    solfa_parser
        .parse(source)
        .map_err(|err| JsError::new(&err.to_string()))
}
