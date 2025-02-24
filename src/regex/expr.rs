#[derive(Debug, Eq, Clone, Copy)]
enum Token {
    Star,
    Opt,
    Plus,
    Concat,
    Alt,
    OpenParenthesis,
    ClosedParenthesis,
    Lit(char),
    CharRange(char, char),
}

impl Token {
    fn precedence(&self) -> u8 {
        match self {
            Token::Star | Token::Plus | Token::Opt => 3,
            Token::Concat => 2,
            Token::Alt => 1,
            _ => 0,
        }
    }
    fn is_op(&self) -> bool {
        !matches!(self, Token::Lit(_) | Token::CharRange(_, _))
    }
    fn to_expr(&self) -> Option<Expr> {
        match self {
            Token::Star => Some(Expr::Star),
            Token::Opt => Some(Expr::Opt),
            Token::Plus => Some(Expr::Plus),
            Token::Concat => Some(Expr::Concat),
            Token::Alt => Some(Expr::Alt),
            Token::Lit(c) => Some(Expr::Literal(*c)),
            Token::CharRange(a, b) => Some(Expr::CharRange(*a, *b)),
            _ => None,
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.precedence() == other.precedence()
    }
}

impl PartialOrd for Token {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.precedence().cmp(&other.precedence()))
    }
}

impl Ord for Token {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.precedence().cmp(&other.precedence())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    Literal(char),
    Concat,
    Alt,
    Star,
    Opt,
    Plus,
    CharRange(char, char),
}

impl Expr {
    pub fn from(c: char) -> Self {
        match c {
            '.' => Expr::Concat,
            '|' => Expr::Alt,
            '*' => Expr::Star,
            '?' => Expr::Opt,
            '+' => Expr::Plus,
            x => Expr::Literal(x),
        }
    }

    fn process_range_token(s: &str) -> Result<Token, String> {
        s.split_once('-')
            .and_then(|(l, r)| Some((l.chars().next()?, r.chars().next()?)))
            .and_then(|(l, r)| Some(Token::CharRange(l, r)))
            .ok_or_else(|| "Invalid range".into())
    }

    fn tokenize(s: &str) -> Result<Vec<Token>, String> {
        s.chars()
            .try_fold((None, Vec::new()), |(mut bracket_buf, mut out), c| {
                match (bracket_buf.as_mut(), c) {
                    (None, '[') => bracket_buf = Some(String::new()),
                    (Some(buf), ']') => {
                        let token = Self::process_range_token(buf)?;
                        out.push(token);
                        bracket_buf = None;
                    }
                    (None, '\\') => bracket_buf = Some(String::from("\\")),
                    (Some(buf), x) if buf == "\\" => {
                        out.push(Token::Lit(x));
                        bracket_buf = None;
                    }
                    (Some(buf), x) => buf.push(x),
                    (None, '(') => out.push(Token::OpenParenthesis),
                    (None, ')') => out.push(Token::ClosedParenthesis),
                    (None, '+') => out.push(Token::Plus),
                    (None, '.') => out.push(Token::Concat),
                    (None, '*') => out.push(Token::Star),
                    (None, '?') => out.push(Token::Opt),
                    (None, '|') => out.push(Token::Alt),
                    (None, x) => out.push(Token::Lit(x)),
                }
                Ok((bracket_buf, out))
            })
            .and_then(|(bracket_buf, out)| {
                if bracket_buf.is_some() {
                    Err("Unclosed '['".into())
                } else {
                    Ok(out)
                }
            })
    }

    fn parse_all(tokens: Vec<Token>) -> Result<Vec<Expr>, String> {
        tokens
            .iter()
            .try_fold((Vec::new(), Vec::new()), |(mut ops, mut out), t| {
                if t.is_op() {
                    match t {
                        Token::OpenParenthesis => ops.push(*t),
                        Token::ClosedParenthesis => {
                            while let Some(op) = ops.pop() {
                                if op == Token::OpenParenthesis {
                                    break;
                                }
                                out.push(op.to_expr().ok_or("Invalid token")?);
                            }
                        }
                        _ => {
                            while ops.last().map_or(false, |op| op.is_op() && op >= t) {
                                out.push(ops.pop().unwrap().to_expr().unwrap());
                            }
                            ops.push(*t);
                        }
                    }
                } else {
                    out.push(t.to_expr().unwrap());
                }
                Ok((ops, out))
            })
            .and_then(|(mut ops, mut out)| {
                while let Some(op) = ops.pop() {
                    if op == Token::OpenParenthesis {
                        return Err("Unmatched '('".into());
                    }
                    out.push(op.to_expr().ok_or("Invalid token")?);
                }
                Ok(out)
            })
    }

    pub fn build(s: &str) -> Result<Vec<Expr>, String> {
        Self::tokenize(s).and_then(Self::parse_all)
    }
}

#[cfg(test)]
mod tests {
    use super::Expr;

    fn run_test(input: &str, expect: &Vec<Expr>) {
        let e = Expr::build(input).unwrap();
        assert_eq!(e, *expect);
    }

    #[test]
    fn test_single_literal() {
        run_test("a", &vec![Expr::Literal('a')]);
    }

    #[test]
    fn test_concat() {
        run_test("ab", &vec![Expr::Literal('a'), Expr::Literal('b')]);
    }

    #[test]
    fn test_concat_operator() {
        run_test(
            "a.b",
            &vec![Expr::Literal('a'), Expr::Literal('b'), Expr::Concat],
        );
    }

    #[test]
    fn test_alt_operator() {
        run_test(
            "a|b",
            &vec![Expr::Literal('a'), Expr::Literal('b'), Expr::Alt],
        );
    }

    #[test]
    fn test_star_operator() {
        run_test("a*", &vec![Expr::Literal('a'), Expr::Star]);
    }

    #[test]
    fn test_opt_operator() {
        run_test("a?", &vec![Expr::Literal('a'), Expr::Opt]);
    }

    #[test]
    fn test_plus_operator() {
        run_test("a+", &vec![Expr::Literal('a'), Expr::Plus]);
    }

    #[test]
    fn test_precedence_and_parentheses() {
        run_test(
            "(a|b).c",
            &vec![
                Expr::Literal('a'),
                Expr::Literal('b'),
                Expr::Alt,
                Expr::Literal('c'),
                Expr::Concat,
            ],
        );
    }

    #[test]
    fn test_complex_expression() {
        run_test(
            "a|(b.c)*",
            &vec![
                Expr::Literal('a'),
                Expr::Literal('b'),
                Expr::Literal('c'),
                Expr::Concat,
                Expr::Star,
                Expr::Alt,
            ],
        );
    }

    #[test]
    fn test_nested_parentheses() {
        run_test(
            "((a|b).c)*",
            &vec![
                Expr::Literal('a'),
                Expr::Literal('b'),
                Expr::Alt,
                Expr::Literal('c'),
                Expr::Concat,
                Expr::Star,
            ],
        );
    }

    #[test]
    fn test_char_range() {
        run_test("[a-z]", &vec![Expr::CharRange('a', 'z')]);
    }

    #[test]
    fn test_complex_char_range() {
        run_test(
            "[a-z]|[A-Z]",
            &vec![
                Expr::CharRange('a', 'z'),
                Expr::CharRange('A', 'Z'),
                Expr::Alt,
            ],
        );
    }

    #[test]
    fn test_num_range() {
        run_test("[0-9]", &vec![Expr::CharRange('0', '9')]);
    }

    #[test]
    fn test_complex_num_range() {
        run_test(
            "[0-9]|[1-9]",
            &vec![
                Expr::CharRange('0', '9'),
                Expr::CharRange('1', '9'),
                Expr::Alt,
            ],
        );
    }
}
