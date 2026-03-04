use solfaml_parser::ast::Note;

#[derive(Debug)]
pub struct StaffLineGroup {
    pub lines: Vec<StaffLine>,
}

#[derive(Debug)]
pub struct StaffLine {
    pub columns: Vec<StaffColumn>,
}

#[derive(Debug)]
pub enum StaffColumn {
    Pulse(Pulse),
    MeasureDivision(MeasureDivision),
    MeasureBar(MeasureBar),
}

#[derive(Debug)]
pub enum MeasureDivision {
    Normal,
    Medium,
}

#[derive(Debug)]
pub struct Pulse {
    pub columns: Vec<PulseColumn>,
}

impl Pulse {
    pub fn new(columns: Vec<PulseColumn>) -> Self {
        Self { columns }
    }

    pub fn empty_note() -> Self {
        Self {
            columns: vec![PulseColumn::EmptyNote],
        }
    }

    pub fn prolonged_note() -> Self {
        Self {
            columns: vec![PulseColumn::ProlongedNote],
        }
    }

    pub fn note(note: Note) -> Self {
        Self {
            columns: vec![PulseColumn::Note(note)],
        }
    }
}

#[derive(Debug)]
pub enum PulseColumn {
    EmptyNote,
    ProlongedNote,
    Note(Note),
    PulseDivision(PulseDivision),
    UnderlineStart(Note),
    UnderlineEnd(Note),
}

#[derive(Debug)]
pub enum PulseDivision {
    Half,
    Quarter,
}

#[derive(Debug)]
pub enum MeasureBar {
    SingleBar,
    DoubleBar,
    RepeatStart,
    RepeatEnd,
}

#[derive(Debug, Clone, Copy)]
pub enum MeasurePosition {
    Start,
    Middle,
    End,
}
