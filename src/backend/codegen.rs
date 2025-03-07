use core::fmt;
use std::{error::Error, io::Write};

use crate::{
    errors::BloggerError,
    parser::parser::{
        ArticleDeclaration, AstNode, List, Paragraph, Program, SectionDeclaration, Statement,
    },
};

pub struct Generator {
    program: Program,
}

impl Generator {
    pub fn new(input: Program) -> Self {
        Self { program: input }
    }

    pub fn compile<'a, W: Write>(&mut self, buf: &'a mut W) -> Result<(), GenerationError> {
        self.program.iter_ast().try_for_each(|node| match node {
            AstNode::Article(v) => Self::generate_article(buf, &v),
            AstNode::Section(v) => Self::generate_section(buf, &v),
            AstNode::Paragraph(v) => Self::generate_paragraph(buf, &v),
            AstNode::Statement(v) => Self::generate_statement(buf, &v),
            AstNode::List(_) => Ok(()),
        })
    }

    fn write_buf<'a, W: Write>(buf: &'a mut W, s: String) -> Result<(), GenerationError> {
        write!(buf, "{}\n", s).map_err(|e| GenerationError::from(e.to_string()))
    }

    fn generate_article<'a, W: Write>(
        buf: &'a mut W,
        article: &ArticleDeclaration,
    ) -> Result<(), GenerationError> {
        Self::write_buf(
            buf,
            format!(r"<h1 className='text-4xl font-bold'>{}</h1>", article.name),
        )
    }

    fn generate_section<'a, W: Write>(
        buf: &'a mut W,
        _: &SectionDeclaration,
    ) -> Result<(), GenerationError> {
        Self::write_buf(buf, "<br/>".to_string())
    }

    fn generate_paragraph<'a, W: Write>(
        buf: &'a mut W,
        _: &Paragraph,
    ) -> Result<(), GenerationError> {
        Self::write_buf(buf, "<br/>".to_string())
    }

    fn generate_statement<'a, W: Write>(
        buf: &'a mut W,
        statement: &Statement,
    ) -> Result<(), GenerationError> {
        match statement {
            Statement::Heading(_, c) => Self::write_buf(
                buf,
                format!("<h3 className='text-3xl'>{}</h3>", c.to_string()),
            ),
            Statement::TextBlock(c) => Self::write_buf(buf, format!("<p>{}</p>", c.to_string())),
            Statement::CodeBlock(c) => Self::write_buf(
                buf,
                format!(
                    r"<pre className='w-full overflow-x-auto'><code>{{`{}`}}</code></pre>",
                    c.to_string()
                ),
            ),
            Statement::Aside(c) => Self::write_buf(
                buf,
                format!(
                    r"
            <div className='p-8 bg-opacity-10 bg-black italic'>
                <p>{}</p>
            </div>
            ",
                    c.to_string()
                ),
            ),
            Statement::List(l) => Self::generate_list(buf, l),
        }
    }

    fn generate_list<'a, W: Write>(buf: &'a mut W, list: &List) -> Result<(), GenerationError> {
        match list {
            List::Ordered(items) => {
                Self::write_buf(
                    buf,
                    format!("<ol className='list-inside list-decimal px-8'>"),
                )?;
                for item in items {
                    Self::write_buf(buf, format!("<li>{}</li>", item))?;
                }
                Self::write_buf(buf, format!("</ol>"))?;
            }
            List::Unordered(items) => {
                Self::write_buf(buf, format!("<ul className='list-disc list-inside px-8'>"))?;
                for item in items {
                    Self::write_buf(buf, format!("<li>{}</li>", item))?;
                }
                Self::write_buf(buf, format!("</ul>"))?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct GenerationError {
    pub msg: String,
}

impl GenerationError {
    fn new(msg: &str) -> Self {
        GenerationError {
            msg: msg.to_string(),
        }
    }
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Compile error: {}", self.msg)
    }
}

impl Error for GenerationError {}

impl From<std::io::Error> for GenerationError {
    fn from(e: std::io::Error) -> Self {
        GenerationError { msg: e.to_string() }
    }
}
impl From<String> for GenerationError {
    fn from(msg: String) -> Self {
        GenerationError { msg }
    }
}

impl From<&str> for GenerationError {
    fn from(msg: &str) -> Self {
        GenerationError::new(msg)
    }
}

impl From<GenerationError> for BloggerError {
    fn from(value: GenerationError) -> Self {
        BloggerError::CodegenError(value.to_string())
    }
}
