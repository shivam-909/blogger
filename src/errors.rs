#[derive(Debug)]
pub enum BloggerError {
    IOError(std::io::Error),
    ParseError(String),
    CodegenError(String),
    RegexError(String),
    LexerError(String),
    CommandError(String),
}

impl std::fmt::Display for BloggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BloggerError::IOError(e) => write!(f, "Blogger Error: IO error: {}", e),
            BloggerError::ParseError(s) => write!(f, "Blogger Error: {}", s),
            BloggerError::CodegenError(s) => {
                write!(f, "Blogger Error: {}", s)
            }
            BloggerError::RegexError(s) => write!(f, "Blogger Error: {}", s),
            BloggerError::LexerError(s) => write!(f, "Blogger Error: {}", s),
            BloggerError::CommandError(s) => write!(f, "Blogger Error: {}", s),
        }
    }
}

impl std::error::Error for BloggerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BloggerError::IOError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for BloggerError {
    fn from(e: std::io::Error) -> Self {
        BloggerError::IOError(e)
    }
}
