use serde::Serialize;
use std::collections::HashMap;

#[cfg(feature = "wasm")]
use {tsify::Tsify, wasm_bindgen::prelude::*};

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct Solfa {
    pub header: HashMap<String, String>,
    pub staffs: Vec<Staff>,
}

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum Dynamic {
    DC { pos: u16 },
    DS { pos: u16 },
    Sign { pos: u16 },
    Accent { pos: u16 },
    Crescendo { start: u16, end: u16 },
    Decrescendo { start: u16, end: u16 },
    Level { pos: u16, kind: DynamicLevel },
}

#[derive(Debug, PartialEq, Serialize)]
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

#[derive(Debug, PartialEq, Serialize)]
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

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum NoteVariant {
    Base,
    Raised,
    Lowered,
}

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum Octave {
    Base,
    Up(u8),
    Down(u8),
}

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct Note {
    pub base: BaseNote,
    pub variant: NoteVariant,
    pub octave: Octave,
}

impl Note {
    pub fn with_octave_up(mut self, value: u8) -> Self {
        self.octave = Octave::Up(value);
        self
    }

    pub fn with_octave_down(mut self, value: u8) -> Self {
        self.octave = Octave::Down(value);
        self
    }
}

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum BeatDivisionKind {
    Medium,
    Normal,
    Half,
    Quarter,
}

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct BeatDivision {
    pub lhs: Box<Measure>,
    pub rhs: Box<Measure>,
    pub kind: BeatDivisionKind,
}

impl BeatDivision {
    pub fn new(kind: BeatDivisionKind, lhs: Measure, rhs: Measure) -> Self {
        BeatDivision {
            lhs: lhs.into(),
            rhs: rhs.into(),
            kind,
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum Measure {
    Blank,
    EmptyNote,
    Note(Note),
    BeatDivision(BeatDivision),
    UnderlinedMeasure(Box<Measure>),
    Repeated(Box<Measure>),
    RepeatStart(Box<Measure>),
    RepeatEnd(Box<Measure>),
}

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct Staff {
    pub dynamics: Vec<Dynamic>,
    pub measures: Vec<Vec<Measure>>,
    pub lyrics: Vec<IndexedLyricsSet>,
}

impl From<StaffPartial> for Staff {
    fn from(value: StaffPartial) -> Self {
        let results = value
            .lines
            .into_iter()
            .enumerate()
            .map(|(idx, line)| {
                let measures = line.measures;
                let lyrics = line.lyrics.map(|ly| IndexedLyricsSet::from((idx, ly)));
                (measures, lyrics)
            })
            .collect::<Vec<_>>();

        let mut lyrics = Vec::new();
        let mut measures = Vec::new();

        for result in results {
            measures.push(result.0);

            if let Some(value) = result.1 {
                lyrics.push(value);
            }
        }

        Self {
            dynamics: value.dynamics.unwrap_or_default(),
            lyrics,
            measures,
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct StaffLine {
    pub measures: Vec<Measure>,
    pub lyrics: Option<Vec<LyricsTree>>,
}

#[derive(Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct StaffPartial {
    pub dynamics: Option<Vec<Dynamic>>,
    pub lines: Vec<StaffLine>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi, namespace))]
pub enum LyricsChunk {
    Placeholder,
    String(String),
    NewLineSuffixed(Box<LyricsChunk>),
    Split(Box<LyricsChunk>, Box<LyricsChunk>),
    Space(Box<LyricsChunk>, Box<LyricsChunk>),
    Concat(Box<LyricsChunk>, Box<LyricsChunk>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[cfg_attr(feature = "wasm", derive(Tsify), tsify(into_wasm_abi))]
pub struct LyricsTree {
    pub prefix: Option<String>,
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
