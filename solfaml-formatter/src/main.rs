use solfaml_formatter::{format_solfa, types::MeasurePosition, visitor::MeasureVisitor};
use solfaml_parser::parse_solfa;

fn main() {
    let source = "
title : hello world
author : foo bar  
key : C
time   : 4/4
tempo: 60
description   : foo
\\bar
---
    | _d .r_ : d . t | m : f ||
    | d : r | m : f ||
    | d : r | m : f ||
    | d : r | m : f ||
";

    let solfa = parse_solfa(source).unwrap_or_else(|err| panic!("{err}"));
    let formatted = format_solfa(&solfa);

    println!("{formatted}")
}
