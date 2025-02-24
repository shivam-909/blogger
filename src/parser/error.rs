use std::error::Error;
use std::fmt;

use crate::{diag::Span, errors::BloggerError, lexer::error::LexerError};

/// ParserError now owns its source code and can render a snippet.
#[derive(Debug)]
pub struct ParserError {
    pub msg: String,
    pub span: Span,
    src: String,
}

impl ParserError {
    pub fn new_with_source<M: Into<String>>(msg: M, span: Span, src: &str) -> Self {
        Self {
            msg: msg.into(),
            span,
            src: src.to_string(),
        }
    }

    pub fn with_source<M: Into<String>>(&self, msg: M, span: Span) -> Self {
        Self::new_with_source(msg, span, &self.src)
    }

    pub fn render(&self) -> String {
        format!("{} at {}", self.msg, self.span.snippet(&self.src))
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Using our render method so that newline and tab characters work.
        write!(f, "Parse error: {}", self.render())
    }
}

impl Error for ParserError {}

impl From<std::io::Error> for ParserError {
    fn from(e: std::io::Error) -> Self {
        ParserError::new_with_source(
            e.to_string(),
            Span::new(Default::default(), Default::default()),
            "",
        )
    }
}

impl From<String> for ParserError {
    fn from(msg: String) -> Self {
        ParserError::new_with_source(msg, Span::new(Default::default(), Default::default()), "")
    }
}

impl From<&str> for ParserError {
    fn from(msg: &str) -> Self {
        ParserError::new_with_source(msg, Span::new(Default::default(), Default::default()), "")
    }
}

impl From<LexerError> for ParserError {
    fn from(value: LexerError) -> Self {
        // Move the source from the lexer error.
        ParserError::new_with_source(value.to_string(), value.span(), &value.src)
    }
}

impl From<&LexerError> for ParserError {
    fn from(value: &LexerError) -> Self {
        ParserError::new_with_source(value.to_string(), value.span(), &value.src)
    }
}

impl From<ParserError> for BloggerError {
    fn from(err: ParserError) -> Self {
        BloggerError::ParseError(err.to_string())
    }
}
