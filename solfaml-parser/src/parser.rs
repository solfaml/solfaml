use std::str::FromStr;

use crate::{
    ast::*,
    error::{ErrorKind, ModalError, ModalResult, ParseError},
    source::{SourceMap, Span},
};

#[derive(Debug)]
pub struct Parser<'a> {
    source: &'a str,
    position: usize,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            errors: vec![],
        }
    }

    fn checkpoint(&self) -> usize {
        self.position
    }

    fn rest(&self) -> &str {
        &self.source[self.position..]
    }

    fn is_eof(&self) -> bool {
        self.rest().is_empty()
    }

    fn peek_char(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn take_char(&mut self, ch: char) -> Option<char> {
        if self.rest().starts_with(ch) {
            self.position += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }

    fn take_while<F>(&mut self, pred: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let result = self
            .rest()
            .chars()
            .take_while(|ch| pred(*ch))
            .collect::<String>();

        self.position += result.len();

        result
    }

    fn skip_whitespace(&mut self) {
        self.take_while(|ch| ch == ' ' || ch == '\t');
    }

    fn skip_multispace(&mut self) {
        self.take_while(char::is_whitespace);
    }

    fn one_of<I>(&mut self, choices: I) -> Option<&str>
    where
        I: IntoIterator<Item = &'static str>,
    {
        let seq = choices.into_iter().find(|seq| self.rest().starts_with(seq));

        if let Some(seq) = seq {
            self.position += seq.len();
        }

        seq
    }

    fn report(&mut self, error_kind: ErrorKind, start_pos: usize) {
        self.errors.push(ParseError {
            span: Span::new(start_pos, self.position),
            kind: error_kind,
        });
    }
}

fn parse_separated<T>(
    p: &mut Parser,
    sep: char,
    mut parser_fn: impl FnMut(&mut Parser) -> ModalResult<T>,
) -> ModalResult<Vec<T>> {
    let mut items = Vec::new();

    loop {
        match parser_fn(p) {
            Ok(item) => items.push(item),
            Err(ModalError::Recover) => {
                p.take_while(|ch| ch != sep);
            }
            Err(e) => return Err(e),
        }

        if p.is_eof() || p.take_char(sep).is_none() {
            break;
        }
    }

    Ok(items)
}

fn parse_measures(p: &mut Parser) -> ModalResult<Vec<Measure>> {
    p.skip_multispace();
    p.take_char('|');

    let start = p.checkpoint();
    let measures = parse_separated(p, '|', parse_base_measure)?;

    if measures.is_empty() {
        p.report(ErrorKind::Expected("at least one measure"), start);
        return Err(ModalError::Recover);
    }

    if p.one_of(["||", "|"]).is_none() {
        p.report(ErrorKind::Expected("`|` or `||`"), p.checkpoint());
    }

    Ok(measures)
}

fn parse_base_measure(p: &mut Parser) -> ModalResult<Measure> {
    let repeat_start = p.take_char(':');
    let body = parse_medium_div(p)?;
    let repeat_end = p.take_char(':');

    let kind = match (repeat_start, repeat_end) {
        (None, None) => MeasureKind::Normal,
        (Some(_), None) => MeasureKind::RepeatStart,
        (None, Some(_)) => MeasureKind::RepeatEnd,
        (Some(_), Some(_)) => MeasureKind::Repeated,
    };

    Ok(Measure { kind, body })
}

fn parse_medium_div(p: &mut Parser) -> ModalResult<MeasureChunk> {
    let lhs = parse_standard_div(p)?;

    if let Some(_) = p.take_char('!') {
        let start = lhs.span.start;
        let rhs = parse_medium_div(p)?;
        let div = MeasureDivision::new(MeasureDivisionKind::Medium, lhs, rhs);

        let chunk = MeasureChunk {
            kind: MeasureChunkKind::Division(div),
            span: Span::new(start, p.checkpoint()),
        };

        Ok(chunk)
    } else {
        Ok(lhs)
    }
}

fn parse_standard_div(p: &mut Parser) -> ModalResult<MeasureChunk> {
    let lhs = parse_half_div(p)?;

    if !p.rest().starts_with(":|")
        && let Some(_) = p.take_char(':')
    {
        let start = lhs.span.start;
        let rhs = parse_standard_div(p)?;
        let div = MeasureDivision::new(MeasureDivisionKind::Standard, lhs, rhs);

        let chunk = MeasureChunk {
            kind: MeasureChunkKind::Division(div),
            span: Span::new(start, p.checkpoint()),
        };

        Ok(chunk)
    } else {
        Ok(lhs)
    }
}

fn parse_half_div(p: &mut Parser) -> ModalResult<MeasureChunk> {
    let lhs = parse_quarter_div(p)?;

    if let Some(_) = p.take_char('.') {
        let start = lhs.span.start;
        let rhs = parse_half_div(p)?;
        let div = MeasureDivision::new(MeasureDivisionKind::Half, lhs, rhs);

        let chunk = MeasureChunk {
            kind: MeasureChunkKind::Division(div),
            span: Span::new(start, p.checkpoint()),
        };

        Ok(chunk)
    } else {
        Ok(lhs)
    }
}

fn parse_quarter_div(p: &mut Parser) -> ModalResult<MeasureChunk> {
    let lhs = parse_base_beat(p)?;

    if let Some(_) = p.take_char(',') {
        let start = lhs.span.start;
        let rhs = parse_quarter_div(p)?;
        let div = MeasureDivision::new(MeasureDivisionKind::Quarter, lhs, rhs);

        let chunk = MeasureChunk {
            kind: MeasureChunkKind::Division(div),
            span: Span::new(start, p.checkpoint()),
        };

        Ok(chunk)
    } else {
        Ok(lhs)
    }
}

fn parse_base_beat(p: &mut Parser) -> ModalResult<MeasureChunk> {
    p.skip_whitespace();

    let start = p.checkpoint();

    let note = if let Some(_) = p.take_char('-') {
        MeasureChunkKind::ProlongedNote
    } else {
        match parse_extended_note(p) {
            Ok(note) => note,
            Err(ModalError::Backtrack) => MeasureChunkKind::EmptyNote,
            Err(err) => return Err(err),
        }
    };

    let chunk = MeasureChunk {
        kind: note,
        span: Span::new(start, p.checkpoint()),
    };

    p.skip_whitespace();

    Ok(chunk)
}

fn parse_extended_note(p: &mut Parser) -> ModalResult<MeasureChunkKind> {
    let start_pos = p.checkpoint();
    let underline_start = p.take_char('_');
    let note = parse_note(p)?;
    let underline_end = p.take_char('_');

    match (underline_start, underline_end) {
        (None, None) => Ok(MeasureChunkKind::Note(note)),
        (Some(_), None) => Ok(MeasureChunkKind::UnderlineStart(note)),
        (None, Some(_)) => Ok(MeasureChunkKind::UnderlineEnd(note)),
        _ => {
            p.report(ErrorKind::InvalidUnderline, start_pos);
            Err(ModalError::Recover)
        }
    }
}

fn parse_note(p: &mut Parser) -> ModalResult<Note> {
    let base = parse_base_note(p)?;
    let variation = pare_note_variation(p);
    let octave = parse_note_octave(p)?;

    Ok(Note {
        base,
        variation,
        octave,
    })
}

fn parse_base_note(p: &mut Parser) -> ModalResult<BaseNote> {
    let note = p.one_of(["d", "r", "m", "f", "s", "l", "t"]);

    match note {
        Some("d") => Ok(BaseNote::D),
        Some("r") => Ok(BaseNote::R),
        Some("m") => Ok(BaseNote::M),
        Some("f") => Ok(BaseNote::F),
        Some("s") => Ok(BaseNote::S),
        Some("l") => Ok(BaseNote::L),
        Some("t") => Ok(BaseNote::T),
        _ => Err(ModalError::Backtrack),
    }
}

fn pare_note_variation(p: &mut Parser) -> Option<NoteVariation> {
    let v = p.one_of(["a", "i"]);

    match v {
        Some("a") => Some(NoteVariation::Lowered),
        Some("i") => Some(NoteVariation::Raised),
        _ => None,
    }
}

fn parse_note_octave(p: &mut Parser) -> ModalResult<i8> {
    let start_pos = p.checkpoint();

    if p.peek_char() == Some(',') {
        let count = p.rest().chars().take_while(|&c| c == ',').count();
        let next_char = p.rest()[count..].trim().chars().next();

        if next_char.is_none() || next_char.is_some_and(|ch| !ch.is_alphabetic()) {
            p.take_while(|ch| ch == ',');
            return resolve_octave(p, -(count as isize), start_pos);
        }
    }

    let value = match p.one_of(["+", "-", "'"]) {
        Some("+") => parse_number(p)?,
        Some("-") => -parse_number::<isize>(p)?,
        Some("'") => 1 + p.take_while(|ch| ch == '\'').len() as isize,
        _ => 0,
    };

    resolve_octave(p, value, start_pos)
}

fn resolve_octave(p: &mut Parser, value: isize, start_pos: usize) -> ModalResult<i8> {
    match value {
        0..=5 => Ok(value as i8),
        _ => {
            p.report(ErrorKind::OctaveOutOfRange, start_pos);
            Err(ModalError::Recover)
        }
    }
}

fn parse_number<T: FromStr>(p: &mut Parser) -> ModalResult<T> {
    let start_pos = p.checkpoint();
    let num = p.take_while(|ch| ch.is_numeric());
    let length = num.len();

    if let Ok(result) = num.parse::<T>() {
        return Ok(result);
    }

    let error = match length {
        0 => ErrorKind::Expected("number"),
        _ => ErrorKind::NumberOutOfRange(num),
    };

    p.report(error, start_pos);

    return Err(ModalError::Recover);
}

#[cfg(test)]
mod tests {
    use crate::parser::{Parser, parse_measures, parse_note};

    // #[test]
    // fn test_measure_parsing() {
    //     let source = "| d : r .  m , f  | s : _l . t_ , - ||";
    //     let measure = parse_measures(&mut Parser::new(source));
    //
    //     insta::assert_debug_snapshot!(measure);
    // }

    #[test]
    fn test_note_parsing() {
        let source = [
            "d", "r", "m", "f", "s", "l", "t", "d'", "r,", "m+2", "f-2", "ti", "da", "ri'", "ma,",
            "si+1", "ra-3", "d,,", "r''",
        ];

        let notes = source
            .into_iter()
            .map(|s| parse_note(&mut Parser::new(s)))
            .collect::<Vec<_>>();

        insta::assert_debug_snapshot!(notes);
    }
}
