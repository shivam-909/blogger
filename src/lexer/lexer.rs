use crate::diag::{Position, Span};

use super::error::LexerError;
use super::{
    error::LexerErrorKind,
    tokens::{Token, TokenKind, TokenSpec},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Block,
}

pub struct Lexer<'a> {
    input: &'a str,
    position: Position,
    specs: Vec<TokenSpec>,
    mode: Mode,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, specs: Vec<TokenSpec>) -> Self {
        Self {
            input,
            position: Position::new(),
            specs,
            mode: Mode::Normal,
        }
    }

    fn next_token(&mut self) -> Option<Result<Token, LexerError>> {
        // Whitespace won't be skipped in TextBlocks
        // because a backtick short circuits normal
        // lexing flow
        self.skip_whitespace();

        // End of input
        if self.position.offset() >= self.input.len() {
            return None;
        }

        Some(match self.mode {
            Mode::Normal => self.lex_normal(),
            Mode::Block => self.lex_block(),
        })
    }

    // lex_normal handles lexing of all tokens that are not text blocks
    // it attempts to find the best match at the current position
    // of the "cursor". If no matches are found, it returns
    // an error.
    fn lex_normal(&mut self) -> Result<Token, LexerError> {
        let start = self.position;
        if let Some((kind, _matched_len)) = self.best_match() {
            if let TokenKind::TextBlock(s) = &kind {
                if s == "`" {
                    self.mode = Mode::Block;
                    return self.lex_block();
                }
            }
            Ok(self.make_token(kind, start, self.position))
        } else {
            let ch = self.peek_char().unwrap();
            Err(LexerError::new(
                LexerErrorKind::UnexpectedChar(ch),
                Span::new(start, self.position),
                self.input,
            ))
        }
    }

    // tokenises a text block, omitting the wrapping backticks
    // and absorbing the internal text.
    fn lex_block(&mut self) -> Result<Token, LexerError> {
        let start = self.position;
        let remaining = &self.input[self.position.offset()..];
        if let Some(rel_end) = remaining.find('`') {
            let text = &remaining[..rel_end];
            // Advance over the block text.
            for _ in 0..text.len() {
                self.advance_char();
            }
            // Consume the backtick.
            self.advance_char();
            self.mode = Mode::Normal;
            Ok(self.make_token(TokenKind::TextBlock(text.to_string()), start, self.position))
        } else {
            Err(LexerError::new(
                LexerErrorKind::UnterminatedBlock,
                Span::new(start, self.position),
                self.input,
            ))
        }
    }

    // Expands the current window until no more matches are found,
    // returning the last match it encountered.
    //
    // Runs in linear time but may be suboptimal in the way the input is handled
    // but source code management in this project is generally quite hacky.
    //
    // TODO: make faster and cleaner?
    fn best_match(&mut self) -> Option<(TokenKind, usize)> {
        let mut candidate = String::new();
        let mut last_match: Option<(TokenKind, usize)> = None;
        let mut chars = self.input[self.position.offset()..].chars().peekable();
        let mut char_count = 0;

        // Keep adding one character at a time until no match is found
        while let Some(&ch) = chars.peek() {
            // Add the next character to our candidate string
            candidate.push(ch);
            char_count += 1;
            chars.next();

            let mut matched = false;
            for spec in &self.specs {
                if let Some(kind) = spec.try_match(&candidate) {
                    last_match = Some((kind.clone(), char_count));
                    matched = true;
                    break;
                }
            }

            if !matched {
                break;
            }
        }

        // Apply the match if we found one
        if let Some((kind, matched_chars)) = last_match {
            // Advance exactly the number of matched characters
            for _ in 0..matched_chars {
                self.advance_char();
            }
            Some((kind, matched_chars))
        } else {
            None
        }
    }

    // helper to create tokens
    fn make_token(&self, kind: TokenKind, start: Position, end: Position) -> Token {
        Token {
            kind,
            span: Span::new(start, end),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if !ch.is_whitespace() {
                break;
            }
            self.advance_char();
        }
    }

    // view the next character in the input without
    // progressing our cursor
    fn peek_char(&self) -> Option<char> {
        self.input[self.position.offset()..].chars().next()
    }

    fn advance_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.position = self.position.advance(ch);
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
