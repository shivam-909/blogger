use crate::{diag::Span, regex::matcher::Matcher};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    Section,
    Article,
    Paragraph,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Heading(String),
    Aside,
    OList,
    UList,
    LItem,
    Code,
    TextBlock(String),
    Ident(String),
}

/// A Token containing its TokenKind plus a Span.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct TokenSpec {
    matcher: Matcher,
    to_kind: fn(&str) -> TokenKind,
}

impl TokenSpec {
    pub fn new(matcher: Matcher, to_kind: fn(&str) -> TokenKind) -> Self {
        Self { matcher, to_kind }
    }

    pub fn try_match(&self, input: &str) -> Option<TokenKind> {
        if self.matcher.matches(input) {
            Some((self.to_kind)(input))
        } else {
            None
        }
    }
}

pub fn token_specs() -> Vec<TokenSpec> {
    vec![
        TokenSpec::new(Matcher::new("\\{").unwrap(), |_| TokenKind::LBrace),
        TokenSpec::new(Matcher::new("\\}").unwrap(), |_| TokenKind::RBrace),
        TokenSpec::new(Matcher::new("\\(").unwrap(), |_| TokenKind::LParen),
        TokenSpec::new(Matcher::new("\\)").unwrap(), |_| TokenKind::RParen),
        TokenSpec::new(Matcher::new("(s.e.c.t.i.o.n)").unwrap(), |_| {
            TokenKind::Section
        }),
        TokenSpec::new(Matcher::new("(a.r.t.i.c.l.e)").unwrap(), |_| {
            TokenKind::Article
        }),
        TokenSpec::new(Matcher::new("(p.a.r.a.g.r.a.p.h)").unwrap(), |_| {
            TokenKind::Paragraph
        }),
        TokenSpec::new(Matcher::new("(h.[1-3])").unwrap(), |s| {
            TokenKind::Heading(s.to_string())
        }),
        TokenSpec::new(Matcher::new("(a.s.i.d.e)").unwrap(), |_| TokenKind::Aside),
        TokenSpec::new(Matcher::new("(o.l)").unwrap(), |_| TokenKind::OList),
        TokenSpec::new(Matcher::new("(u.l)").unwrap(), |_| TokenKind::UList),
        TokenSpec::new(Matcher::new("(l.i)").unwrap(), |_| TokenKind::LItem),
        TokenSpec::new(Matcher::new("(c.o.d.e)").unwrap(), |_| TokenKind::Code),
        TokenSpec::new(Matcher::new("(`)").unwrap(), |s| {
            TokenKind::TextBlock(s.to_string())
        }),
        TokenSpec::new(Matcher::new("(([a-z]|[A-Z]|[0-9])*)").unwrap(), |s| {
            TokenKind::Ident(s.to_string())
        }),
    ]
}
