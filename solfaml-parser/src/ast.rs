use serde::Serialize;
use std::{collections::BTreeMap, str::FromStr};

use crate::source::Span;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct Solfa {
    // pub header: Header,
    pub staffs: Vec<Staff>,
}

// #[derive(Debug)]
// pub struct KeyValuePair<T> {
//     span_key: Span,
//     span_val: Span,
//     value: T,
// }
//
// #[derive(Debug, PartialEq, Serialize)]
// #[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
// pub struct Header {
//     pub title: Option<String>,
//     pub author: Option<String>,
//     pub time: Option<Time>,
//     pub key: Option<Key>,
//     pub description: Option<String>,
//     pub tempo: Option<usize>,
//     pub vocals: Option<usize>,
//     pub extra: BTreeMap<String, String>,
// }
//
// #[derive(Debug, PartialEq, Serialize)]
// #[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
// pub struct Time {
//     pub top: usize,
//     pub bottom: usize,
// }
//
// impl FromStr for Time {
//     type Err = Error;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let (top, bottom) = s
//             .split_once("/")
//             .and_then(|(top, bottom)| top.parse().ok().zip(bottom.parse().ok()))
//             .ok_or(Error::InvalidTime(s.to_string()))?;
//
//         Ok(Self { top, bottom })
//     }
// }
//
// impl std::fmt::Display for Time {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}/{}", self.top, self.bottom)
//     }
// }
//
// #[derive(Debug, PartialEq, Serialize)]
// #[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
// pub enum Key {
//     C,
//     G,
//     D,
//     A,
//     E,
//     B,
//     #[serde(rename = "F#")]
//     Fs,
//     #[serde(rename = "C#")]
//     Cs,
//     F,
//     Bb,
//     Eb,
//     Ab,
//     Db,
//     Gb,
//     Cb,
// }
//
// impl FromStr for Key {
//     type Err = Error;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "C" => Ok(Self::C),
//             "D" => Ok(Self::D),
//             "A" => Ok(Self::A),
//             "E" => Ok(Self::E),
//             "B" => Ok(Self::B),
//             "F#" => Ok(Self::Fs),
//             "C#" => Ok(Self::Cs),
//             "Bb" => Ok(Self::Bb),
//             "Eb" => Ok(Self::Eb),
//             "Ab" => Ok(Self::Ab),
//             "Db" => Ok(Self::Db),
//             "Gb" => Ok(Self::Gb),
//             "Cb" => Ok(Self::Cb),
//             _ => Err(Error::InvalidKey(s.to_string())),
//         }
//     }
// }
//
// impl std::fmt::Display for Key {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Key::Fs => write!(f, "F#"),
//             Key::Cs => write!(f, "C#"),
//             _ => write!(f, "{self:?}"),
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub struct Dynamic {
    pub kind: DynamicKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum DynamicKind {
    DC,
    DS,
    Sign,
    Accent,
    CrescendoStart,
    DecrescendoStart,
    CrescendoEnd,
    DecrescendoEnd,
    Level(DynamicLevel),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum DynamicLevel {
    FFF,
    FF,
    F,
    MF,
    MP,
    P,
    PP,
    PPP,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum BaseNote {
    D,
    R,
    M,
    F,
    S,
    L,
    T,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum NoteVariation {
    Base,
    Raised,
    Lowered,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct Note {
    pub base: BaseNote,
    pub variation: Option<NoteVariation>,
    pub octave: i8,
}

// impl std::fmt::Display for Note {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let base = format!("{:?}", self.base).to_lowercase();
//
//         let modifier = match self.variant {
//             NoteVariant::Base => "",
//             NoteVariant::Raised => "i",
//             NoteVariant::Lowered => "a",
//         };
//
//         let suffix = match self.octave {
//             Octave::Base => "".to_string(),
//             Octave::Up(1) => "'".to_string(),
//             Octave::Down(1) => ",".to_string(),
//             Octave::Up(value) => format!("+{value}"),
//             Octave::Down(value) => format!("-{value}"),
//         };
//
//         write!(f, "{base}{modifier}{suffix}")
//     }
// }

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum MeasureDivisionKind {
    Medium,
    Standard,
    Half,
    Quarter,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct MeasureDivision {
    pub lhs: Box<MeasureChunk>,
    pub rhs: Box<MeasureChunk>,
    pub kind: MeasureDivisionKind,
}

impl MeasureDivision {
    pub fn new(kind: MeasureDivisionKind, lhs: MeasureChunk, rhs: MeasureChunk) -> Self {
        MeasureDivision {
            lhs: lhs.into(),
            rhs: rhs.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct StaffLine {
    pub measures: Vec<Measure>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct Measure {
    pub kind: MeasureKind,
    pub body: MeasureChunk,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum MeasureKind {
    Normal,
    Repeated,
    RepeatStart,
    RepeatEnd,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub struct MeasureChunk {
    pub kind: MeasureChunkKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum MeasureChunkKind {
    EmptyNote,
    ProlongedNote,
    Note(Note),
    NoteGroup(Vec<Note>),
    Division(MeasureDivision),
    UnderlineStart(Note),
    UnderlineEnd(Note),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct Staff {
    pub dynamics: Vec<Dynamic>,
    pub lines: Vec<StaffLine>,
    pub lyrics: Vec<IndexedLyricsSet>,
}

impl Staff {
    pub fn new(dynamics: Vec<Dynamic>, lines: Vec<StaffLinePartial>) -> Self {
        let results = lines
            .into_iter()
            .enumerate()
            .map(|(idx, line)| {
                let measures = line.measures;
                let lyrics = line.lyrics.map(|ly| IndexedLyricsSet::from((idx, ly)));
                (measures, lyrics)
            })
            .collect::<Vec<_>>();

        let mut lyrics = Vec::new();
        let mut lines = Vec::new();

        for (measures, lyrics_set) in results {
            lines.push(StaffLine { measures });

            if let Some(value) = lyrics_set {
                lyrics.push(value);
            }
        }

        Self {
            dynamics,
            lyrics,
            lines,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct StaffLinePartial {
    pub measures: Vec<Measure>,
    pub lyrics: Option<Vec<LyricsTree>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub struct LyricsChunk {
    pub kind: LyricsChunkKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum LyricsChunkKind {
    Placeholder,
    String(String),
    LineBreak(Box<LyricsChunk>, Box<LyricsChunk>),
    Split(Box<LyricsChunk>, Box<LyricsChunk>),
    Space(Box<LyricsChunk>, Box<LyricsChunk>),
    Concat(Box<LyricsChunk>, Box<LyricsChunk>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct LyricsPrefix {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct LyricsTree {
    pub prefix: Option<LyricsPrefix>,
    pub root: LyricsChunk,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct IndexedLyricsSet {
    pub index: usize,
    pub lyrics: Vec<LyricsTree>,
}

impl From<(usize, Vec<LyricsTree>)> for IndexedLyricsSet {
    fn from((index, lyrics): (usize, Vec<LyricsTree>)) -> Self {
        Self { index, lyrics }
    }
}
