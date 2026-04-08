use std::ops::{Bound, RangeBounds};

use crate::source::Span;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Expected(String),
    UnexpectedChar(char),
    UnexpectedEOF,
    NumberOutOfRange(String),
    OctaveOutOfRange,
    InvalidUnderline,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Expected(seq) => write!(f, "expected {seq}"),
            ErrorKind::UnexpectedChar(ch) => write!(f, "unexpected chracter '{ch}'"),
            ErrorKind::UnexpectedEOF => write!(f, "unexpected end of file"),
            ErrorKind::NumberOutOfRange(num) => write!(f, "number out of range: '{num}'"),
            ErrorKind::OctaveOutOfRange => write!(f, "octave out of the valid range"),
            ErrorKind::InvalidUnderline => write!(f, "invalid underline"),
        }
    }
}

impl ErrorKind {
    pub fn expected(msg: &str) -> Self {
        Self::Expected(msg.to_string())
    }

    pub fn expected_bounds<B: RangeBounds<usize>>(bounds: B) -> Self {
        let min = match bounds.start_bound() {
            Bound::Included(n) => n.to_string(),
            Bound::Excluded(n) => (n + 1).to_string(),
            Bound::Unbounded => "0".to_string(),
        };

        let msg = match bounds.end_bound() {
            Bound::Included(n) => format!("between {} and {} items", min, n),
            Bound::Excluded(n) => format!("between {} and {} items", min, n - 1),
            Bound::Unbounded => format!("at least {} items", min),
        };

        Self::Expected(msg)
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub span: Span,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ModalError {
    Recover,
    Backtrack,
    Cut,
}

pub type ModalResult<T> = std::result::Result<T, ModalError>;
