use std::fmt;

use crate::{diag::Span, errors::BloggerError};

#[derive(Debug, Clone)]
pub enum LexerErrorKind {
    UnexpectedChar(char),
    UnterminatedBlock,
    UnexpectedEOF,
}

#[derive(Debug, Clone)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub span: Span,
    pub src: String,
}

impl LexerError {
    pub fn new(kind: LexerErrorKind, span: Span, src: &str) -> Self {
        Self {
            kind,
            span,
            src: src.to_string(),
        }
    }

    fn render(&self) -> String {
        let snippet = self.span.snippet(&self.src);
        match &self.kind {
            LexerErrorKind::UnexpectedChar(c) => {
                format!("Unexpected character '{}' at: {}", c, snippet)
            }
            LexerErrorKind::UnterminatedBlock => format!("Unterminated block\n{}", snippet),
            LexerErrorKind::UnexpectedEOF => "Unexpected EOF".to_string(),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lexer Error: {}", self.render())
    }
}

impl std::error::Error for LexerError {}

impl From<LexerError> for BloggerError {
    fn from(value: LexerError) -> Self {
        BloggerError::LexerError(value.to_string())
    }
}
