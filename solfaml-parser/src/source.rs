use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub col: usize,
    pub byte_offset: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug)]
pub struct SourceMap<'a> {
    pub source: &'a str,
    pub lines: Vec<usize>,
}

impl<'a> SourceMap<'a> {
    pub fn new(source: &'a str) -> Self {
        let lines = source
            .char_indices()
            .flat_map(|(i, ch)| if ch == '\n' { Some(i) } else { None })
            .collect();

        Self { source, lines }
    }

    pub fn location(&self, byte_offset: usize) -> Location {
        let line = self.lines.partition_point(|&start| start <= byte_offset);
        let line_start = self.lines[line];
        let col = self.source[line_start..byte_offset].chars().count();

        Location {
            line,
            col,
            byte_offset,
        }
    }
}
