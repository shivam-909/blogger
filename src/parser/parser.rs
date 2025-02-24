use std::collections::HashMap;

use super::error::ParserError;
use crate::diag::Span;
use crate::lexer::lexer::Lexer;
use crate::lexer::tokens::{Token, TokenKind};

// Program is represented as a tree
#[derive(Debug)]
pub struct Program {
    pub article: ArticleDeclaration,
    pub sections: HashMap<String, SectionDeclaration>,
}

impl Program {
    // iter_ast returns an iterator that traverses in the order of program declaration
    // i.e: starts at the article, then each section entirely, in the order it is called
    // in the article
    pub fn iter_ast(&self) -> ASTIterator {
        ASTIterator::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct ArticleDeclaration {
    pub name: String,
    pub section_calls: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SectionDeclaration {
    pub name: String,
    pub paragraphs: Vec<Paragraph>,
}

#[derive(Debug, Clone)]
pub struct Paragraph {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Heading(String, String),
    TextBlock(String),
    CodeBlock(String),
    Aside(String),
    List(List),
}

#[derive(Debug, Clone)]
pub enum List {
    Ordered(Vec<String>),
    Unordered(Vec<String>),
}

#[derive(Debug, Clone, Copy)]
pub enum AstNode<'a> {
    Article(&'a ArticleDeclaration),
    Section(&'a SectionDeclaration),
    Paragraph(&'a Paragraph),
    Statement(&'a Statement),
    List(&'a List),
}

impl<'a> AstNode<'a> {
    pub fn children(&self, program: &'a Program) -> Vec<AstNode<'a>> {
        match self {
            AstNode::Article(article) => article
                .section_calls
                .iter()
                .filter_map(|name| program.sections.get(name).map(AstNode::Section))
                .collect(),
            AstNode::Section(section) => {
                section.paragraphs.iter().map(AstNode::Paragraph).collect()
            }
            AstNode::Paragraph(paragraph) => paragraph
                .statements
                .iter()
                .map(AstNode::Statement)
                .collect(),
            AstNode::Statement(stmt) => match stmt {
                Statement::List(list) => vec![AstNode::List(list)],
                _ => vec![],
            },
            AstNode::List(_) => vec![],
        }
    }
}

pub struct ASTIterator<'a> {
    program: &'a Program,
    stack: Vec<AstNode<'a>>,
}

impl<'a> ASTIterator<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self {
            program,
            stack: vec![AstNode::Article(&program.article)],
        }
    }
}

impl<'a> Iterator for ASTIterator<'a> {
    type Item = AstNode<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|node| {
            node.children(self.program)
                .into_iter()
                .rev()
                .for_each(|child| self.stack.push(child));
            node
        })
    }
}

/// Parser consumes tokens produced by the Lexer (each Token holds a TokenKind and its Span)
/// and holds a reference to the full source for error rendering.
pub struct Parser<'a> {
    tokens: std::iter::Peekable<Lexer<'a>>,
    source: &'a String,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>, source: &'a String) -> Self {
        Self {
            tokens: lexer.peekable(),
            source,
        }
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut article_opt: Option<ArticleDeclaration> = None;
        let mut sections = HashMap::new();

        while let Some(token) = self.peek_token()? {
            let t = token.clone();
            match t.kind {
                TokenKind::Article => {
                    if article_opt.is_some() {
                        return Err(ParserError::new_with_source(
                            "Multiple article declarations found",
                            t.span,
                            self.source,
                        ));
                    }
                    article_opt = Some(self.parse_article_declaration()?);
                }
                TokenKind::Section => {
                    let sec = self.parse_section_declaration()?;
                    if sections.contains_key(&sec.name) {
                        return Err(ParserError::new_with_source(
                            format!("Duplicate section: {}", sec.name),
                            t.span,
                            self.source,
                        ));
                    }
                    sections.insert(sec.name.clone(), sec);
                }
                _ => {
                    return Err(ParserError::new_with_source(
                        format!("Unexpected token at program level: {:?}", token),
                        t.span,
                        self.source,
                    ))
                }
            }
        }

        let article = article_opt.ok_or_else(|| {
            ParserError::new_with_source(
                "Missing article declaration",
                Span::new(Default::default(), Default::default()),
                self.source,
            )
        })?;
        Ok(Program { article, sections })
    }

    fn parse_article_declaration(&mut self) -> Result<ArticleDeclaration, ParserError> {
        self.expect_token(TokenKind::Article)?;
        // Allow an optional article name.
        let name = match self.peek_token()? {
            Some(token) if token.kind == TokenKind::LBrace => String::new(),
            _ => self.expect_ident()?,
        };
        self.expect_token(TokenKind::LBrace)?;
        let section_calls = self.parse_until(TokenKind::RBrace, Self::expect_ident_dynamic)?;
        self.expect_token(TokenKind::RBrace)?;
        Ok(ArticleDeclaration {
            name,
            section_calls,
        })
    }

    fn parse_section_declaration(&mut self) -> Result<SectionDeclaration, ParserError> {
        self.expect_token(TokenKind::Section)?;
        let name = self.expect_ident()?;
        self.expect_token(TokenKind::LBrace)?;
        let paragraphs = self.parse_until(TokenKind::RBrace, Self::parse_paragraph)?;
        self.expect_token(TokenKind::RBrace)?;
        Ok(SectionDeclaration { name, paragraphs })
    }

    fn parse_paragraph(&mut self) -> Result<Paragraph, ParserError> {
        self.expect_token(TokenKind::Paragraph)?;
        self.expect_token(TokenKind::LBrace)?;
        let statements = self.parse_until(TokenKind::RBrace, Self::parse_statement)?;
        self.expect_token(TokenKind::RBrace)?;
        Ok(Paragraph { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match self.peek_token()? {
            Some(token) if matches!(token.kind, TokenKind::Heading(_)) => {
                let heading_token = self.next_token()?;
                let heading_type = if let Token {
                    kind: TokenKind::Heading(ref h),
                    ..
                } = heading_token
                {
                    h.clone()
                } else {
                    unreachable!()
                };
                self.expect_token(TokenKind::LBrace)?;
                let content = self.parse_heading_content()?;
                self.expect_token(TokenKind::RBrace)?;
                Ok(Statement::Heading(heading_type, content))
            }
            Some(token) if matches!(token.kind, TokenKind::TextBlock(_)) => {
                let tb_token = self.next_token()?;
                if let Token {
                    kind: TokenKind::TextBlock(text),
                    ..
                } = tb_token
                {
                    Ok(Statement::TextBlock(text))
                } else {
                    unreachable!()
                }
            }
            Some(token) if token.kind == TokenKind::Code => {
                let code_token = self.next_token()?;
                if code_token.kind != TokenKind::Code {
                    unreachable!()
                }
                self.expect_token(TokenKind::LBrace)?;
                let tb_token = self.next_token()?;
                self.expect_token(TokenKind::RBrace)?;
                match tb_token.kind {
                    TokenKind::TextBlock(code_text) => Ok(Statement::CodeBlock(code_text)),
                    _ => Err(ParserError::new_with_source(
                        "Expected text block inside code block",
                        tb_token.span,
                        self.source,
                    )),
                }
            }
            Some(token) if token.kind == TokenKind::Aside => self.parse_aside(),
            Some(token) if matches!(token.kind, TokenKind::OList | TokenKind::UList) => {
                let list = self.parse_list()?;
                Ok(Statement::List(list))
            }
            Some(token) => Err(ParserError::new_with_source(
                format!("Unexpected token in statement: {:?}", token),
                token.span,
                self.source,
            )),
            None => Err(ParserError::new_with_source(
                "Unexpected end of input while parsing statement",
                Span::new(Default::default(), Default::default()),
                self.source,
            )),
        }
    }

    fn parse_heading_content(&mut self) -> Result<String, ParserError> {
        let token = self.next_token()?;
        match token.kind {
            TokenKind::Ident(text) | TokenKind::TextBlock(text) => Ok(text),
            other => Err(ParserError::new_with_source(
                format!("Expected heading content, found {:?}", other),
                token.span,
                self.source,
            )),
        }
    }

    fn parse_aside(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(TokenKind::Aside)?;
        self.expect_token(TokenKind::LBrace)?;
        let token = self.next_token()?;
        let content = match token.kind {
            TokenKind::TextBlock(text) | TokenKind::Ident(text) => text,
            other => {
                return Err(ParserError::new_with_source(
                    format!("Expected TextBlock or Ident in aside, found {:?}", other),
                    token.span,
                    self.source,
                ))
            }
        };
        self.expect_token(TokenKind::RBrace)?;
        Ok(Statement::Aside(content))
    }

    fn parse_list(&mut self) -> Result<List, ParserError> {
        let list_token = self.next_token()?;
        let is_ordered = match list_token.kind {
            TokenKind::OList => true,
            TokenKind::UList => false,
            _ => {
                return Err(ParserError::new_with_source(
                    format!("Expected OList or UList, found {:?}", list_token),
                    list_token.span,
                    self.source,
                ))
            }
        };
        self.expect_token(TokenKind::LBrace)?;
        let items = self.parse_until(TokenKind::RBrace, Self::parse_list_item)?;
        self.expect_token(TokenKind::RBrace)?;
        Ok(if is_ordered {
            List::Ordered(items)
        } else {
            List::Unordered(items)
        })
    }

    fn parse_list_item(&mut self) -> Result<String, ParserError> {
        self.expect_token(TokenKind::LItem)?;
        self.expect_token(TokenKind::LBrace)?;
        let token = self.next_token()?;
        let item = match token.kind {
            TokenKind::TextBlock(text) | TokenKind::Ident(text) => text,
            other => {
                return Err(ParserError::new_with_source(
                    format!(
                        "Expected TextBlock or Ident in list item, found {:?}",
                        other
                    ),
                    token.span,
                    self.source,
                ))
            }
        };
        self.expect_token(TokenKind::RBrace)?;
        Ok(item)
    }

    fn parse_until<F, T>(&mut self, end: TokenKind, f: F) -> Result<Vec<T>, ParserError>
    where
        F: Fn(&mut Self) -> Result<T, ParserError>,
    {
        let mut items = Vec::new();
        while let Some(token) = self.peek_token()? {
            if token.kind == end {
                break;
            }
            items.push(f(self)?);
        }
        Ok(items)
    }

    fn expect_ident_dynamic(&mut self) -> Result<String, ParserError> {
        self.expect_ident()
    }

    fn expect_token(&mut self, expected: TokenKind) -> Result<(), ParserError> {
        let token = self.next_token()?;
        if token.kind == expected {
            Ok(())
        } else {
            Err(ParserError::new_with_source(
                format!("Expected {:?} but found {:?}", expected, token.kind),
                token.span,
                self.source,
            ))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParserError> {
        let token = self.next_token()?;
        match token.kind {
            TokenKind::Ident(s) => Ok(s),
            other => Err(ParserError::new_with_source(
                format!("Expected identifier, found {:?}", other),
                token.span,
                self.source,
            )),
        }
    }

    fn peek_token(&mut self) -> Result<Option<&Token>, ParserError> {
        match self.tokens.peek() {
            Some(Ok(token)) => Ok(Some(token)),
            Some(Err(e)) => Err(e.clone().into()),
            None => Ok(None),
        }
    }

    fn next_token(&mut self) -> Result<Token, ParserError> {
        match self.tokens.next() {
            Some(Ok(token)) => Ok(token),
            Some(Err(e)) => Err(e.into()),
            None => Err(ParserError::new_with_source(
                "Unexpected end of input",
                Span::new(Default::default(), Default::default()),
                self.source,
            )),
        }
    }
}
