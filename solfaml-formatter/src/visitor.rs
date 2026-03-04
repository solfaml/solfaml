use solfaml_parser::ast;

use crate::types::{
    MeasureBar, MeasureDivision, MeasurePosition, Pulse, PulseColumn, PulseDivision, StaffColumn,
};

pub trait MeasureVisitor {
    fn visit_measure(&self, position: MeasurePosition, columns: &mut Vec<StaffColumn>);
}

pub trait MeasureChunkVisitor {
    fn visit_chunk(&self, columns: &mut Vec<StaffColumn>);
}

pub trait PulseVisitor {
    fn visit_pulse(&self, columns: &mut Vec<PulseColumn>);
}

impl MeasureVisitor for ast::Measure {
    fn visit_measure(&self, position: MeasurePosition, columns: &mut Vec<StaffColumn>) {
        let start_bar = match (position, self.kind) {
            (_, ast::MeasureKind::RepeatStart | ast::MeasureKind::Repeated) => {
                Some(MeasureBar::RepeatStart)
            }
            (MeasurePosition::Start, _) => Some(MeasureBar::DoubleBar),
            (MeasurePosition::Middle | MeasurePosition::End, _) => Some(MeasureBar::SingleBar),
        };

        if let Some(bar) = start_bar {
            columns.push(StaffColumn::MeasureBar(bar));
        }

        self.root.visit_chunk(columns);

        let end_bar = match (position, self.kind) {
            (MeasurePosition::End, ast::MeasureKind::RepeatEnd | ast::MeasureKind::Repeated) => {
                Some(MeasureBar::RepeatEnd)
            }
            (MeasurePosition::End, _) => Some(MeasureBar::DoubleBar),
            _ => None,
        };

        if let Some(bar) = end_bar {
            columns.push(StaffColumn::MeasureBar(bar));
        }
    }
}

impl MeasureChunkVisitor for ast::MeasureChunk {
    fn visit_chunk(&self, columns: &mut Vec<StaffColumn>) {
        match self {
            ast::MeasureChunk::Division(division) => division.visit_chunk(columns),
            ast::MeasureChunk::EmptyNote => columns.push(StaffColumn::Pulse(Pulse::empty_note())),
            ast::MeasureChunk::ProlongedNote => {
                columns.push(StaffColumn::Pulse(Pulse::prolonged_note()))
            }
            ast::MeasureChunk::Note(note) => {
                columns.push(StaffColumn::Pulse(Pulse::note(note.clone())))
            }
            ast::MeasureChunk::UnderlineStart(note) => todo!(),
            ast::MeasureChunk::UnderlineEnd(note) => todo!(),
            ast::MeasureChunk::UnderlinedNote(note) => todo!(),
        }
    }
}

impl MeasureChunkVisitor for ast::MeasureDivision {
    fn visit_chunk(&self, columns: &mut Vec<StaffColumn>) {
        match self.kind {
            ast::MeasureDivisionKind::Medium | ast::MeasureDivisionKind::Normal => {
                self.lhs.visit_chunk(columns);

                let division = match self.kind {
                    ast::MeasureDivisionKind::Medium => MeasureDivision::Medium,
                    ast::MeasureDivisionKind::Normal => MeasureDivision::Normal,
                    _ => unreachable!(),
                };

                columns.push(StaffColumn::MeasureDivision(division));

                self.rhs.visit_chunk(columns);
            }
            ast::MeasureDivisionKind::Half | ast::MeasureDivisionKind::Quarter => {
                let mut pulse = Vec::new();

                self.visit_pulse(&mut pulse);

                columns.push(StaffColumn::Pulse(Pulse::new(pulse)));
            }
        }
    }
}

impl PulseVisitor for ast::MeasureChunk {
    fn visit_pulse(&self, columns: &mut Vec<PulseColumn>) {
        match self {
            ast::MeasureChunk::EmptyNote => columns.push(PulseColumn::EmptyNote),
            ast::MeasureChunk::ProlongedNote => columns.push(PulseColumn::ProlongedNote),
            ast::MeasureChunk::Note(note) => columns.push(PulseColumn::Note(note.clone())),
            ast::MeasureChunk::Division(division) => division.visit_pulse(columns),
            ast::MeasureChunk::UnderlineStart(note) => todo!(),
            ast::MeasureChunk::UnderlineEnd(note) => todo!(),
            ast::MeasureChunk::UnderlinedNote(note) => todo!(),
        }
    }
}

impl PulseVisitor for ast::MeasureDivision {
    fn visit_pulse(&self, columns: &mut Vec<PulseColumn>) {
        self.lhs.visit_pulse(columns);

        let division = match self.kind {
            ast::MeasureDivisionKind::Half => PulseDivision::Half,
            ast::MeasureDivisionKind::Quarter => PulseDivision::Quarter,
            _ => unreachable!(),
        };

        columns.push(PulseColumn::PulseDivision(division));

        self.rhs.visit_pulse(columns);
    }
}
