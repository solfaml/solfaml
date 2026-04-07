use crate::source::Span;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Expected(&'static str),
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

#[derive(Debug)]
pub struct ParseError {
    pub span: Span,
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ModalError {
    Recover,
    Backtrack,
}

pub type ModalResult<T> = std::result::Result<T, ModalError>;
