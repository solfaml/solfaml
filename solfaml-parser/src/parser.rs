use std::{ops::RangeBounds, str::FromStr};

use crate::{
    ast::*,
    error::{ErrorKind, ModalError, ModalResult, ParseError},
    source::Span,
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

    fn reset(&mut self, position: usize) {
        self.position = position;
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

    fn recover_with_default<T: Default>(&mut self, symbol: char) -> T {
        self.take_while(|ch| ch != symbol);
        T::default()
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

fn parse_separated<S, T: std::fmt::Debug>(
    p: &mut Parser,
    bounds: impl RangeBounds<usize>,
    mut sep_fn: impl FnMut(&mut Parser) -> ModalResult<S>,
    mut parser_fn: impl FnMut(&mut Parser) -> ModalResult<T>,
) -> ModalResult<Vec<T>> {
    let mut items = Vec::new();
    let mut checkpoint = p.checkpoint();
    let start_pos = p.checkpoint();

    'outer: loop {
        match parser_fn(p) {
            Ok(item) => {
                items.push(item);
                checkpoint = p.checkpoint();
            }
            Err(ModalError::Recover) => {
                while let Some(next) = p.peek_char() {
                    match sep_fn(p) {
                        Ok(_) => continue 'outer,
                        Err(ModalError::Cut) => break,
                        Err(_) => p.take_char(next),
                    };
                }
            }
            Err(_) => {
                p.reset(checkpoint);
                break;
            }
        }

        if sep_fn(p).is_err() || p.is_eof() {
            break;
        }
    }

    if bounds.contains(&items.len()) {
        Ok(items)
    } else {
        p.report(ErrorKind::expected_bounds(bounds), start_pos);
        Err(ModalError::Recover)
    }
}

fn measure_sep(p: &mut Parser) -> ModalResult<()> {
    if p.rest().starts_with("||") || p.rest().starts_with('\n') {
        return Err(ModalError::Cut);
    }

    match p.take_char('|') {
        Some(_) => Ok(()),
        _ => Err(ModalError::Cut),
    }
}

fn multiline_sep(p: &mut Parser) -> ModalResult<()> {
    match p.take_while(char::is_whitespace).contains('\n') {
        true => Ok(()),
        false => Err(ModalError::Cut),
    }
}

fn parse_solfa(p: &mut Parser) -> ModalResult<Solfa> {
    p.skip_multispace();

    let staffs = parse_separated(p, 1.., multiline_sep, parse_staff)?;

    p.skip_multispace();

    Ok(Solfa { staffs })
}

fn parse_staff(p: &mut Parser) -> ModalResult<Staff> {
    let dynamics = parse_dynamics(p).unwrap_or_else(|_| p.recover_with_default('\n'));
    let lines = parse_separated(p, 4..=4, multiline_sep, parse_staff_line)?;

    Ok(Staff::new(dynamics, lines))
}

fn parse_dynamics(p: &mut Parser) -> ModalResult<Vec<Dynamic>> {
    if p.one_of(["|!"]).is_none() {
        return Ok(Vec::new());
    }

    p.skip_whitespace();

    let dynamics = parse_separated(
        p,
        ..,
        |p| {
            p.skip_whitespace();
            Ok(())
        },
        parse_base_dynamic,
    );

    if p.one_of(["||", "|"]).is_none() {
        p.report(ErrorKind::expected("`|` or `||`"), p.checkpoint());
    }

    dynamics
}

fn parse_base_dynamic(p: &mut Parser) -> ModalResult<Dynamic> {
    let start_pos = p.checkpoint();

    let d = p.one_of([
        "fff", "ff", "f", "mf", "mp", "ppp", "pp", "p", "DC", "DS", "$", "^", "<<", "<", ">>", ">",
    ]);

    let kind = match d {
        Some("DC") => DynamicKind::DC,
        Some("DS") => DynamicKind::DS,
        Some("$") => DynamicKind::Sign,
        Some("^") => DynamicKind::Accent,
        Some("<") => DynamicKind::CrescendoStart,
        Some(">") => DynamicKind::DecrescendoStart,
        Some("<<") => DynamicKind::CrescendoEnd,
        Some(">>") => DynamicKind::DecrescendoEnd,
        Some("fff") => DynamicKind::Level(DynamicLevel::FFF),
        Some("ff") => DynamicKind::Level(DynamicLevel::FF),
        Some("f") => DynamicKind::Level(DynamicLevel::F),
        Some("mf") => DynamicKind::Level(DynamicLevel::MF),
        Some("mp") => DynamicKind::Level(DynamicLevel::MP),
        Some("p") => DynamicKind::Level(DynamicLevel::P),
        Some("pp") => DynamicKind::Level(DynamicLevel::PP),
        Some("ppp") => DynamicKind::Level(DynamicLevel::PPP),
        _ => return Err(ModalError::Cut),
    };

    Ok(Dynamic {
        kind,
        span: Span::new(start_pos, p.checkpoint()),
    })
}

fn parse_staff_line(p: &mut Parser) -> ModalResult<StaffLinePartial> {
    let measures = parse_measures(p).unwrap_or_else(|_| p.recover_with_default('\n'));

    let lyrics = if p.rest().trim_start().starts_with("|") {
        None
    } else {
        p.skip_multispace();
        parse_lyrics_lines(p).ok()
    };

    Ok(StaffLinePartial { measures, lyrics })
}

fn parse_measures(p: &mut Parser) -> ModalResult<Vec<Measure>> {
    p.skip_multispace();

    if p.one_of(["||", "|"]).is_none() {
        p.report(ErrorKind::expected("`|` or `||`"), p.checkpoint());
    }

    let start = p.checkpoint();
    let measures = parse_separated(p, .., measure_sep, parse_base_measure)?;

    if measures.is_empty() {
        p.report(ErrorKind::expected("at least one measure"), start);
        return Err(ModalError::Recover);
    }

    if p.one_of(["||", "|"]).is_none() {
        p.report(ErrorKind::expected("`|` or `||`"), p.checkpoint());
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

    if p.take_char('!').is_none() {
        return Ok(lhs);
    };

    let start_pos = lhs.span.start;
    let rhs = parse_medium_div(p)?;
    let div = MeasureDivision::new(MeasureDivisionKind::Medium, lhs, rhs);

    let chunk = MeasureChunk {
        kind: MeasureChunkKind::Division(div),
        span: Span::new(start_pos, p.checkpoint()),
    };

    Ok(chunk)
}

fn parse_standard_div(p: &mut Parser) -> ModalResult<MeasureChunk> {
    let lhs = parse_half_div(p)?;

    if !p.rest().starts_with(":|") && p.take_char(':').is_some() {
        let start_pos = lhs.span.start;
        let rhs = parse_standard_div(p)?;
        let div = MeasureDivision::new(MeasureDivisionKind::Standard, lhs, rhs);

        let chunk = MeasureChunk {
            kind: MeasureChunkKind::Division(div),
            span: Span::new(start_pos, p.checkpoint()),
        };

        Ok(chunk)
    } else {
        Ok(lhs)
    }
}

fn parse_half_div(p: &mut Parser) -> ModalResult<MeasureChunk> {
    let lhs = parse_quarter_div(p)?;

    if p.take_char('.').is_none() {
        return Ok(lhs);
    };

    let start_pos = lhs.span.start;
    let rhs = parse_half_div(p)?;
    let div = MeasureDivision::new(MeasureDivisionKind::Half, lhs, rhs);

    let chunk = MeasureChunk {
        kind: MeasureChunkKind::Division(div),
        span: Span::new(start_pos, p.checkpoint()),
    };

    Ok(chunk)
}

fn parse_quarter_div(p: &mut Parser) -> ModalResult<MeasureChunk> {
    let lhs = parse_base_beat(p)?;

    if p.take_char(',').is_none() {
        return Ok(lhs);
    };

    let start_pos = lhs.span.start;
    let rhs = parse_quarter_div(p)?;
    let div = MeasureDivision::new(MeasureDivisionKind::Quarter, lhs, rhs);

    let chunk = MeasureChunk {
        kind: MeasureChunkKind::Division(div),
        span: Span::new(start_pos, p.checkpoint()),
    };

    Ok(chunk)
}

fn parse_base_beat(p: &mut Parser) -> ModalResult<MeasureChunk> {
    p.skip_whitespace();

    let start = p.checkpoint();

    let note = if p.take_char('-').is_some() {
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
    match p.one_of(["d", "r", "m", "f", "s", "l", "t"]) {
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
    match p.one_of(["a", "i"]) {
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
        -5..=5 => Ok(value as i8),
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
        0 => ErrorKind::expected("number"),
        _ => ErrorKind::NumberOutOfRange(num),
    };

    p.report(error, start_pos);

    return Err(ModalError::Recover);
}

fn parse_lyrics_lines(p: &mut Parser) -> ModalResult<Vec<LyricsTree>> {
    return parse_separated(p, 1.., multiline_sep, parse_lyrics_tree);
}

fn parse_lyrics_tree(p: &mut Parser) -> ModalResult<LyricsTree> {
    let mut prefix = None;

    if p.take_char('(').is_some() {
        let start_pos = p.checkpoint();
        let value = p.take_while(|ch| ch != ')' && ch != '\n');
        let span = Span::new(start_pos, p.checkpoint());

        prefix = Some(LyricsPrefix { value, span });

        if p.take_char(')').is_none() {
            p.report(ErrorKind::expected("`)`"), p.checkpoint());
            return Err(ModalError::Recover);
        }
    }

    let root = parse_lyrics_break(p)?;

    Ok(LyricsTree { prefix, root })
}

fn parse_lyrics_break(p: &mut Parser) -> ModalResult<LyricsChunk> {
    let lhs = parse_lyrics_join(p)?;

    p.skip_whitespace();

    if p.take_char('\\').is_none() {
        return Ok(lhs);
    };

    if let Ok(rhs) = parse_lyrics_break(p) {
        Ok(LyricsChunk {
            span: Span::new(lhs.span.start, p.checkpoint()),
            kind: LyricsChunkKind::LineBreak(lhs.into(), rhs.into()),
        })
    } else {
        p.report(ErrorKind::expected("right-handed side"), p.checkpoint());
        Err(ModalError::Recover)
    }
}

fn parse_lyrics_join(p: &mut Parser) -> ModalResult<LyricsChunk> {
    p.skip_whitespace();

    let lhs = parse_lyrics_split(p)?;
    let trimmed = p.rest().trim_start();

    let constructor = if trimmed.starts_with('_') {
        p.skip_whitespace();
        p.take_char('_');
        LyricsChunkKind::Concat
    } else if p.rest().starts_with(' ') && !trimmed.starts_with('\\') {
        p.take_char(' ');
        LyricsChunkKind::Space
    } else {
        return Ok(lhs);
    };

    if let Ok(rhs) = parse_lyrics_join(p) {
        Ok(LyricsChunk {
            span: Span::new(lhs.span.start, p.checkpoint()),
            kind: constructor(lhs.into(), rhs.into()),
        })
    } else {
        p.report(ErrorKind::expected("right-handed side"), p.checkpoint());
        Err(ModalError::Recover)
    }
}

fn parse_lyrics_split(p: &mut Parser) -> ModalResult<LyricsChunk> {
    let lhs = parse_lyrics_string(p)?;

    if p.take_char('/').is_none() {
        return Ok(lhs);
    };

    if let Ok(rhs) = parse_lyrics_string(p) {
        Ok(LyricsChunk {
            span: Span::new(lhs.span.start, p.checkpoint()),
            kind: LyricsChunkKind::Split(lhs.into(), rhs.into()),
        })
    } else {
        p.report(ErrorKind::expected("right-handed side"), p.checkpoint());
        Err(ModalError::Recover)
    }
}

fn parse_lyrics_string(p: &mut Parser) -> ModalResult<LyricsChunk> {
    let start_pos = p.checkpoint();

    if p.take_char('$').is_some() {
        return Ok(LyricsChunk {
            kind: LyricsChunkKind::Placeholder,
            span: Span::new(start_pos, p.checkpoint()),
        });
    }

    let lyrics = p.take_while(|ch| !" _|$\n/\\".contains(ch));

    if lyrics.is_empty() {
        Err(ModalError::Backtrack)
    } else {
        Ok(LyricsChunk {
            span: Span::new(start_pos, p.checkpoint()),
            kind: LyricsChunkKind::String(lyrics),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        Parser, parse_lyrics_lines, parse_lyrics_tree, parse_measures, parse_note, parse_solfa,
        parse_staff,
    };

    // #[test]
    // fn text_measure_parsing() {
    //     let source = "| : .d | d : r .  m , f  | s : _l . t_ , - ||";
    //     let measure = parse_measures(&mut Parser::new(source));
    //
    //     insta::assert_debug_snapshot!(measure);
    // }
    //
    // #[test]
    // fn test_note_parsing() {
    //     let source = [
    //         "d", "r", "m", "f", "s", "l", "t", "d'", "r,", "m+2", "f-2", "ti", "da", "ri'", "ma,",
    //         "si+1", "ra-3", "d,,", "r''",
    //     ];
    //
    //     let notes = source
    //         .into_iter()
    //         .map(|s| parse_note(&mut Parser::new(s)))
    //         .collect::<Vec<_>>();
    //
    //     insta::assert_debug_snapshot!(notes);
    // }

    #[test]
    fn test_simple_solfa_parsing() {
        let source = "
| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||

| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||";

        let mut parser = Parser::new(source);
        let result = parse_solfa(&mut parser);

        insta::assert_debug_snapshot!(result);
    }
}
