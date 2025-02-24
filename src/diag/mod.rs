#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    offset: usize,
    line: usize,
    column: usize,
}

impl Position {
    pub fn new() -> Self {
        Self {
            offset: 0,
            line: 0,
            column: 0,
        }
    }
    pub fn advance(&self, ch: char) -> Self {
        Self {
            offset: self.offset + ch.len_utf8(),
            line: if ch == '\n' { self.line + 1 } else { self.line },
            column: if ch == '\n' { 0 } else { self.column + 1 },
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    start: Position,
    end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
    pub fn snippet(&self, src: &str) -> String {
        let line = src.lines().nth(self.start.line).unwrap_or("").trim_start();
        let underline: String = (0..line.len())
            .map(|i| {
                if i >= self.start.column && i <= self.end.column {
                    '^'
                } else {
                    '-'
                }
            })
            .collect();
        format!(
            "\nLine: {}, Column: {}\n>> '{}'\n   {}",
            self.start.line, self.start.column, line, underline
        )
    }
}
