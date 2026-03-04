use std::collections::BTreeMap;

use super::ast::*;

use winnow::{
    ascii::{alphanumeric1, digit1, multispace0, multispace1, space0, space1},
    combinator::{alt, delimited, not, opt, peek, repeat, separated, seq},
    prelude::*,
    token::{one_of, take_until, take_while},
};

pub const DEFAULT_VOCAL_LINES: usize = 4;

pub fn solfa_parser(input: &mut &str) -> ModalResult<Solfa> {
    seq! {
        Solfa {
            _: multispace0,
            header: header_parser,
            _: multispace0,
            _: "---",
            _: multispace1,
            staffs: separated(1.., |input: &mut &str| staff_parser(input, &header), multispace1),
            _: multispace0,
        }
    }
    .parse_next(input)
}

pub fn header_parser(input: &mut &str) -> ModalResult<Header> {
    separated(
        0..,
        seq! (
            alphanumeric1.map(|s: &str| s.to_string()),
            _: space0,
            _: ":",
            _: space0,
            metadata_value_parser,
        ),
        '\n',
    )
    .try_map(|metadata: BTreeMap<_, _>| Header::try_from(metadata))
    .parse_next(input)
}

pub fn metadata_value_parser(input: &mut &str) -> ModalResult<String> {
    seq!(
        take_while(1.., |ch: char| ch != '\n').map(|s: &str| s.to_string()),
        repeat(
            0..,
            seq!(
                _: '\n',
                _:space0,
                _: '\\',
                _:space0,
                take_while(1.., |ch: char| ch != '\n').map(|s: &str| s.to_string())
            )
            .map(|(seq,)| seq)
        )
        .map(|acc: Vec<String>| acc)
    )
    .map(|(first, rest)| {
        [first]
            .into_iter()
            .chain(rest)
            .collect::<Vec<_>>()
            .join("\n")
    })
    .parse_next(input)
}

pub fn staff_parser(input: &mut &str, header: &Header) -> ModalResult<Staff> {
    let vocals = header.vocals.unwrap_or(DEFAULT_VOCAL_LINES);

    seq! {
        StaffPartial {
            dynamics: opt(seq!(dynamics_parser, _: "\n")).map(|d| d.map(|(d,)| d)),
            _: opt(seq!(staff_bar_parser, "\n")),
            lines: separated(vocals, staff_line_parser, multispace1)
        }
    }
    .map(Staff::from)
    .parse_next(input)
}

pub fn staff_bar_parser(input: &mut &str) -> ModalResult<()> {
    seq!(
        _: "|",
        _: take_while(1.., |ch: char| ch == '-'),
        _: alt(("||", "|")),
    )
    .parse_next(input)
}

pub fn staff_line_parser(input: &mut &str) -> ModalResult<StaffLinePartial> {
    seq!(StaffLinePartial {
        measures: measure_parser,
        lyrics: opt(seq!(_: multispace1, lyrics_parser).map(|(l,)| l)),
    })
    .parse_next(input)
}

pub fn dynamics_parser(input: &mut &str) -> ModalResult<Vec<Dynamic>> {
    seq!(
        _: "|:",
        _: space0,
        separated(0.., dynamic_base_parser, space0),
        _: space0,
        _: alt(("||", "|")),
    )
    .map(|(d,)| d)
    .parse_next(input)
}

pub fn dynamic_base_parser(input: &mut &str) -> ModalResult<Dynamic> {
    alt((
        dynamic_level_parser,
        seq!(_: "DC", _: space0, pos_parser).map(|(pos,)| Dynamic::DC { pos }),
        seq!(_: "DS", _: space0, pos_parser).map(|(pos,)| Dynamic::DS { pos }),
        seq!(_: "$", _: space0, pos_parser).map(|(pos,)| Dynamic::Sign { pos }),
        seq!(_: "^", _: space0, pos_parser).map(|(pos,)| Dynamic::Accent { pos }),
        seq!(_: "<", _: space0, range_parser)
            .map(|((start, end),)| Dynamic::Crescendo { start, end }),
        seq!(_: ">", _: space0, range_parser)
            .map(|((start, end),)| Dynamic::Decrescendo { start, end }),
    ))
    .parse_next(input)
}

pub fn dynamic_level_parser(input: &mut &str) -> ModalResult<Dynamic> {
    seq!(
        alt((
            "fff".map(|_| DynamicLevel::FFF),
            "ff".map(|_| DynamicLevel::FF),
            "f" .map(|_| DynamicLevel::F),
            "mf".map(|_| DynamicLevel::MF),
            "mp".map(|_| DynamicLevel::MP),
            "p" .map(|_| DynamicLevel::P),
            "pp".map(|_| DynamicLevel::PP),
            "ppp".map(|_| DynamicLevel::PPP),
        )),
        _: space0,
        pos_parser
    )
    .map(|(kind, pos)| Dynamic::Level { kind, pos })
    .parse_next(input)
}

pub fn pos_parser(input: &mut &str) -> ModalResult<u16> {
    seq!(_: "[", _: space0, digit1, _: space0, _: "]")
        .try_map(|(pos,): (&str,)| pos.parse())
        .parse_next(input)
}

pub fn range_parser(input: &mut &str) -> ModalResult<(u16, u16)> {
    seq!(_: "[", _: space0, digit1,_: ",", digit1, _: space0, _: "]")
        .try_map(|(start, end): (&str, &str)| {
            start.parse().and_then(|s| end.parse().map(|e| (s, e)))
        })
        .parse_next(input)
}

pub fn measure_parser(input: &mut &str) -> ModalResult<Vec<Measure>> {
    seq!(
        _: multispace0,
        _: opt("|"),
        separated(1.., measure_base_parser, "|"),
        _: alt(("||", "|")),
    )
    .map(|(m,)| m)
    .parse_next(input)
}

pub fn measure_base_parser(input: &mut &str) -> ModalResult<Measure> {
    seq!(opt(":"), medium_div_parser, opt(":"),)
        .map(|(rep_start, root, rep_end)| {
            let kind = match (rep_start, rep_end) {
                (Some(_), Some(_)) => MeasureKind::Repeated,
                (Some(_), None) => MeasureKind::RepeatStart,
                (None, Some(_)) => MeasureKind::RepeatEnd,
                (None, None) => MeasureKind::Normal,
            };

            Measure { kind, root }
        })
        .parse_next(input)
}

pub fn medium_div_parser(input: &mut &str) -> ModalResult<MeasureChunk> {
    seq!(standard_div_parser, opt(seq!(_: "!", medium_div_parser)))
        .map(|(lhs, rhs)| match rhs {
            Some((rhs,)) => {
                MeasureChunk::Division(MeasureDivision::new(MeasureDivisionKind::Medium, lhs, rhs))
            }
            None => lhs,
        })
        .parse_next(input)
}

pub fn standard_div_parser(input: &mut &str) -> ModalResult<MeasureChunk> {
    seq!(half_div_parser, opt(seq!(_: ":", standard_div_parser)))
        .map(|(lhs, rhs)| match rhs {
            Some((rhs,)) => {
                MeasureChunk::Division(MeasureDivision::new(MeasureDivisionKind::Normal, lhs, rhs))
            }
            None => lhs,
        })
        .parse_next(input)
}

pub fn half_div_parser(input: &mut &str) -> ModalResult<MeasureChunk> {
    seq!(quarter_div_parser, opt(seq!(_: ".", half_div_parser)))
        .map(|(lhs, rhs)| match rhs {
            Some((rhs,)) => {
                MeasureChunk::Division(MeasureDivision::new(MeasureDivisionKind::Half, lhs, rhs))
            }
            None => lhs,
        })
        .parse_next(input)
}

pub fn quarter_div_parser(input: &mut &str) -> ModalResult<MeasureChunk> {
    seq!(
        alt((blank_parser, base_beat_parser)),
        opt(seq!(_: ",", quarter_div_parser))
    )
    .map(|(lhs, rhs)| match rhs {
        Some((rhs,)) => {
            MeasureChunk::Division(MeasureDivision::new(MeasureDivisionKind::Quarter, lhs, rhs))
        }
        _ => lhs,
    })
    .parse_next(input)
}

pub fn base_beat_parser(input: &mut &str) -> ModalResult<MeasureChunk> {
    seq!(
        _: space0,
        alt((
            "-".map(|_| MeasureChunk::ProlongedNote),
            extended_note_parser,
        )),
        _: space0,
    )
    .map(|(b,)| b)
    .parse_next(input)
}

pub fn extended_note_parser(input: &mut &str) -> ModalResult<MeasureChunk> {
    seq!(
        opt(seq!('_', _: space0)),
        note_parser,
        opt(seq!(_: space0, '_')),
    )
    .map(|(l, n, r)| match (l, r) {
        (None, None) => MeasureChunk::Note(n),
        (Some(_), None) => MeasureChunk::UnderlineStart(n),
        (None, Some(_)) => MeasureChunk::UnderlineEnd(n),
        (Some(_), Some(_)) => MeasureChunk::UnderlinedNote(n),
    })
    .parse_next(input)
}

pub fn blank_parser(input: &mut &str) -> ModalResult<MeasureChunk> {
    seq!(space1, peek(alt((".", ":", "!", "|"))))
        .map(|_| MeasureChunk::EmptyNote)
        .parse_next(input)
}

pub fn note_parser(input: &mut &str) -> ModalResult<Note> {
    seq! {
        Note {
            base: base_note_parser,
            variant: opt(one_of(('a', 'i'))).map(|v| match v {
                Some('a') => NoteVariant::Lowered,
                Some('i') => NoteVariant::Raised,
                _ => NoteVariant::Base,
            }),
            octave: opt(octave_parser).map(|o| o.unwrap_or(Octave::Base))
        }
    }
    .parse_next(input)
}

pub fn base_note_parser(input: &mut &str) -> ModalResult<BaseNote> {
    one_of(('d', 'r', 'm', 'f', 's', 'l', 't'))
        .map(|note| match note {
            'd' => BaseNote::D,
            'r' => BaseNote::R,
            'm' => BaseNote::M,
            'f' => BaseNote::F,
            's' => BaseNote::S,
            'l' => BaseNote::L,
            't' => BaseNote::T,
            _ => unreachable!(),
        })
        .parse_next(input)
}

pub fn octave_parser(input: &mut &str) -> ModalResult<Octave> {
    alt((
        seq!(_: "+", digit1)
            .try_map(|(d,): (&str,)| d.parse())
            .map(Octave::Up),
        seq!(_: "-", digit1)
            .try_map(|(d,): (&str,)| d.parse())
            .map(Octave::Down),
        seq!(repeat(1.., ','), _: not(seq!(_: space0, base_note_parser)))
            .try_map(|(s,): (Vec<char>,)| s.len().try_into())
            .map(Octave::Down),
        repeat(1.., '\'')
            .try_map(|s: Vec<char>| s.len().try_into())
            .map(Octave::Up),
    ))
    .parse_next(input)
}

pub fn lyrics_parser(input: &mut &str) -> ModalResult<Vec<LyricsTree>> {
    separated(1.., lyrics_tree_parser, multispace1).parse_next(input)
}

pub fn lyrics_tree_parser(input: &mut &str) -> ModalResult<LyricsTree> {
    seq! {
        LyricsTree {
            prefix: opt(
                delimited(
                    "(",
                    take_until(1.., ")").map(|s: &str| s.to_string()),
                    ")"
                )
                .map(|p| p)
            ),
            root: lyrics_chunk_parser,
        }
    }
    .parse_next(input)
}

pub fn lyrics_chunk_parser(input: &mut &str) -> ModalResult<LyricsChunk> {
    seq!(
        _: space0,
        base_lyrics_parser,
        opt(seq!(
            alt((
                seq!(' ', _: space0, _: not('_')),
                seq!(_: space0, '_'),
            ))
            .map(|(c,)| c),
            lyrics_chunk_parser
        )),
        _: space0,
    )
    .map(|(lhs, rhs)| match rhs {
        Some((sep, rhs)) => match sep {
            '_' => LyricsChunk::Concat(Box::new(lhs), rhs.into()),
            ' ' => LyricsChunk::Space(Box::new(lhs), rhs.into()),
            _ => unreachable!(),
        },
        None => lhs,
    })
    .parse_next(input)
}

pub fn base_lyrics_parser(input: &mut &str) -> ModalResult<LyricsChunk> {
    seq!(
        alt((
            "$".map(|_| LyricsChunk::Placeholder),
            seq!(
                take_while(1.., |ch: char| !" _|$\n/\\".contains(ch)),
                opt(seq!(_: "/", base_lyrics_parser)),
            )
            .map(|(lhs, rhs)| {
                let lhs = LyricsChunk::String(lhs.to_string());
                match rhs {
                    Some((rhs,)) => LyricsChunk::Split(Box::new(lhs), Box::new(rhs)),
                    None => lhs,
                }
            }),
        )),
        opt("\\")
    )
    .map(|(lyrics, newline)| match newline.is_some() {
        false => lyrics,
        true => LyricsChunk::NewLineSuffixed(Box::new(lyrics)),
    })
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use crate::parser::{
        dynamics_parser, header_parser, lyrics_tree_parser, measure_parser, note_parser,
        solfa_parser,
    };

    #[test]
    fn test_header_parser() {
        let source = "title: foo
author: bar
time: 4/4
key: C
description: Hello World!
  \\ Lorem Ipsum";

        let metadata = header_parser.parse(source);

        insta::assert_debug_snapshot!(metadata);
    }

    #[test]
    fn test_dynamics_parsing() {
        let source = "|: f[1] <[3,7] ^[8] mp[10] ||";
        let dynamics = dynamics_parser.parse(source).unwrap();

        insta::assert_debug_snapshot!(dynamics);
    }

    #[test]
    fn test_note_parsing() {
        let source = [
            "d", "r", "m", "f", "s", "l", "t", "d'", "r,", "m+2", "f-2", "ti", "da", "ri'", "ma,",
            "si+1", "ra-3", "d,,", "r''",
        ];

        let notes = source
            .into_iter()
            .map(|s| note_parser.parse(s))
            .collect::<Vec<_>>();

        insta::assert_debug_snapshot!(notes);
    }

    #[test]
    fn test_measure_parsing() {
        let source = "| : | d : r .  m , f  | s : _l . t_ , - ||";
        let measure = measure_parser.parse(source);

        insta::assert_debug_snapshot!(measure);
    }

    #[test]
    fn test_lyrics_parsing() {
        let source = "(1.) do re_mi\\ fasola ti/e do $";
        let lyrics = lyrics_tree_parser.parse(source);

        insta::assert_debug_snapshot!(lyrics);
    }

    #[test]
    fn test_simple_solfa_parsing() {
        let source = "
---
| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||
";

        let result = solfa_parser.parse(source);

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_per_voice_lyrics_parsing() {
        let source = "
---
| d : r ||
| d : r ||
(>) do re
| d : r ||
| d : r ||
(>) doo ree
";

        let result = solfa_parser.parse(source);

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_multi_staff_parsing() {
        let source = "
---
| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||
| d : r | m : f ||

(>) do re  mi fa

| s : l | t : d' ||
| s : l | t : d' ||

(>) so la  ti do

| s : l | t : - ||
| s : l | t : - ||

(>) so la  ti
";

        let result = solfa_parser.parse(source);

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_measure_repetition_parsing() {
        let source = "
---
| d : r |: m : f | s : l :| t : d' ||
| d : r |: m : f | s : l :| t : d' ||
| d : r |: m : f | s : l :| t : d' ||
| d : r |: m : f | s : l :| t : d' ||
";

        let result = solfa_parser.parse(source);

        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_full_parsing() {
        let source = "
title: foo
author: bar
time: 4/4
key: C
description: Hello World!

---

|: p[1]       <[4,7]             ^[8]  DC[9] ||
|--------------------------------------------||
|  d : r : m | f . s , l : t  | _d'_ : ri+2  ||
|  d : r : m | f . s , l : t  | _d'_ : ri+2  ||
|  d : r : m | f . s , l : t  | _d'_ : ra-1  ||
|  d : r : m | f . s , l : t  |  d,  : ra-1  ||

(1.) do re_mi   fasola    ti/e   do     re
(2.) do re_mi   fasola    ti/e   do     $
";

        let solfa = solfa_parser
            .parse(source)
            .unwrap_or_else(|err| panic!("{}", err.to_string()));

        insta::assert_debug_snapshot!(solfa);
    }
}
