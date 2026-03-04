use solfaml_parser::ast::{Note, NoteVariant, Octave};

use crate::types::{MeasureBar, MeasureDivision, PulseColumn, PulseDivision};

pub trait Displayable: std::fmt::Display {
    fn output_len(&self) -> u8;
}

impl Displayable for Note {
    fn output_len(&self) -> u8 {
        let mut length = 1;

        match self.variant {
            NoteVariant::Base => {}
            _ => length += 1,
        }

        match self.octave {
            Octave::Up(value) | Octave::Down(value) => length += value.min(2),
            _ => {}
        }

        length
    }
}

impl Displayable for PulseDivision {
    fn output_len(&self) -> u8 {
        1
    }
}

impl Displayable for MeasureDivision {
    fn output_len(&self) -> u8 {
        1
    }
}

impl Displayable for MeasureBar {
    fn output_len(&self) -> u8 {
        match self {
            MeasureBar::SingleBar => 1,
            MeasureBar::RepeatStart | MeasureBar::RepeatEnd | MeasureBar::DoubleBar => 2,
        }
    }
}

impl std::fmt::Display for MeasureDivision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeasureDivision::Normal => write!(f, ":"),
            MeasureDivision::Medium => write!(f, "!"),
        }
    }
}

impl std::fmt::Display for PulseDivision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PulseDivision::Half => write!(f, "."),
            PulseDivision::Quarter => write!(f, ","),
        }
    }
}

impl std::fmt::Display for MeasureBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeasureBar::SingleBar => write!(f, "|"),
            MeasureBar::DoubleBar => write!(f, "||"),
            MeasureBar::RepeatStart => write!(f, "|:"),
            MeasureBar::RepeatEnd => write!(f, ":|"),
        }
    }
}

impl std::fmt::Display for PulseColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PulseColumn::EmptyNote => write!(f, " "),
            PulseColumn::ProlongedNote => write!(f, "-"),
            PulseColumn::Note(note) => write!(f, "{note}"),
            PulseColumn::PulseDivision(division) => write!(f, "{division}"),
            PulseColumn::UnderlineStart(note) => todo!(),
            PulseColumn::UnderlineEnd(note) => todo!(),
        }
    }
}
