pub mod display;
pub mod types;
pub mod visitor;

use solfaml_parser::ast::{Header, Solfa, Staff};

use crate::{
    types::{MeasurePosition, StaffLine, StaffLineGroup},
    visitor::MeasureVisitor,
};

pub fn format_solfa(ast: &Solfa) -> String {
    let header = format_header(&ast.header);
    let staffs = format_staffs(&ast.staffs);

    format!("{header}\n\n---\n\n{staffs}")
}

fn format_header(header: &Header) -> String {
    let lines = [
        ("title", header.title.as_ref().map(|t| t.to_string())),
        ("author", header.author.as_ref().map(|a| a.to_string())),
        ("time", header.time.as_ref().map(|t| t.to_string())),
        ("key", header.key.as_ref().map(|k| k.to_string())),
        ("tempo", header.tempo.map(|t| t.to_string())),
        ("vocals", header.vocals.map(|v| v.to_string())),
        (
            "description",
            header.description.as_ref().map(|d| d.to_string()),
        ),
    ]
    .into_iter()
    .filter_map(|(k, v)| v.map(|v| (k, v)))
    .chain(
        header
            .extra
            .iter()
            .map(|(k, v)| (k.as_str(), v.to_string())),
    );

    lines
        .map(|(k, v)| format!("{k}: {}", v.replace("\n", "\n  \\ ").trim_end()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_staffs(staffs: &[Staff]) -> String {
    let flattened = flatten_staffs(staffs);

    let mut staffs_buffer = Vec::new();

    for group in flattened {
        let mut staff_buffer = Vec::new();

        for staff_line in group.lines {
            let mut line_buffer = Vec::new();

            for col in staff_line.columns {
                let s = match col {
                    types::StaffColumn::Pulse(pulse) => pulse
                        .columns
                        .into_iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                    types::StaffColumn::MeasureDivision(division) => division.to_string(),
                    types::StaffColumn::MeasureBar(bar) => bar.to_string(),
                };

                line_buffer.push(s);
            }

            staff_buffer.push(line_buffer.join(" "));
        }

        staffs_buffer.push(staff_buffer.join("\n"))
    }

    staffs_buffer.join("\n\n")
}

fn flatten_staffs(staffs: &[Staff]) -> Vec<StaffLineGroup> {
    let mut results = Vec::new();

    for (i, staff) in staffs.iter().enumerate() {
        let mut lines = Vec::new();

        for line in &staff.lines {
            let mut columns = Vec::new();

            for (j, measure) in line.measures.iter().enumerate() {
                let position = if i == 0 && j == 0 {
                    MeasurePosition::Start
                } else if i == staffs.len() - 1 && j == line.measures.len() - 1 {
                    MeasurePosition::End
                } else {
                    MeasurePosition::Middle
                };

                measure.visit_measure(position, &mut columns);
            }

            lines.push(StaffLine { columns });
        }

        results.push(StaffLineGroup { lines });
    }

    results
}
