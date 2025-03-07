use std::{collections::HashMap, fmt::Debug, sync::Mutex};

use super::expr::Expr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Condition {
    Id(char),
    CharClass(Vec<char>),
}

impl Condition {
    pub fn to_string(&self) -> String {
        match self {
            Self::Id(c) => c.to_string(),
            Self::CharClass(chars) => format!("{chars:?}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum State {
    Transition {
        id: usize,
        condition: Condition,
        output: Option<usize>,
    },
    Split {
        id: usize,
        left: Option<usize>,
        right: Option<usize>,
    },
    Accept {
        id: usize,
    },
}

impl State {
    pub fn to_string(&self) -> String {
        match self {
            Self::Transition {
                condition, output, ..
            } => {
                format!("[match '{}' -> {output:?}]", condition.to_string())
            }
            Self::Split { left, right, .. } => format!("[-> ({left:?} | {right:?})]"),
            Self::Accept { .. } => "[accept]".to_string(),
        }
    }

    pub fn set_out(&mut self, next_state: Option<usize>) {
        match self {
            Self::Transition { output, .. } => *output = next_state,
            Self::Split { left, right, .. } => {
                if left.is_none() && right.is_none() {
                    *left = next_state;
                }
                *right = next_state;
            }
            _ => {}
        }
    }

    pub fn get_id(&self) -> usize {
        match self {
            Self::Transition { id, .. } => *id,
            Self::Split { id, .. } => *id,
            Self::Accept { id } => *id,
        }
    }

    pub fn matches_condition(&self, ch: char) -> bool {
        match self {
            Self::Transition { condition, .. } => match condition {
                Condition::Id(c) => *c == ch,
                Condition::CharClass(v) => v.contains(&ch),
            },
            _ => false,
        }
    }
}

#[derive(Debug)]
struct Fragment {
    head: usize,
    out: Vec<usize>,
}

impl Fragment {
    fn detached(head: usize) -> Self {
        Self {
            head,
            out: vec![head],
        }
    }
    fn single_link(head: usize, out: usize) -> Self {
        Self {
            head,
            out: vec![out],
        }
    }
    fn multi_link(head: usize, left: Vec<usize>, right: Vec<usize>) -> Self {
        let mut outs = left;
        outs.extend(right);
        Self { head, out: outs }
    }
}

#[derive(Debug)]
pub struct NFA {
    head: usize,
    state_list: Vec<State>,
}

impl NFA {
    pub fn new() -> Self {
        Self {
            head: 0,
            state_list: Vec::new(),
        }
    }

    fn add_state(&mut self, state: State) -> usize {
        self.state_list.push(state.clone());
        let idx = self.state_list.len() - 1;

        idx
    }

    fn link_state(&mut self, f_idx: usize, t_idx: usize) -> Result<(), String> {
        self.state_list[f_idx].set_out(Some(t_idx));
        let mut updated = self.state_list[f_idx].clone();
        updated.set_out(Some(t_idx));
        Ok(())
    }

    fn link_fragment(&mut self, frag: &mut Fragment, t_idx: usize) -> Result<(), String> {
        frag.out
            .iter()
            .try_for_each(|&idx| self.link_state(idx, t_idx))
    }

    fn link_fragments(&mut self, from: &mut Fragment, to: Fragment) -> Result<(), String> {
        self.link_fragment(from, to.head)?;
        from.out.iter_mut().for_each(|o| *o = to.head);
        Ok(())
    }

    fn range_chars(start: char, end: char) -> Result<Vec<char>, String> {
        (start <= end)
            .then(|| {
                (start as u32..=end as u32)
                    .filter_map(std::char::from_u32)
                    .collect()
            })
            .ok_or_else(|| "Ranges must be specified in ascending order".into())
    }

    pub fn build(expr: Vec<Expr>) -> Result<Self, String> {
        let mut nfa = Self::new();
        let mut stack = Vec::new();
        let mut counter = 0;

        for e in expr {
            match e {
                Expr::Literal(c) => {
                    let st = State::Transition {
                        id: counter,
                        condition: Condition::Id(c),
                        output: None,
                    };
                    let idx = nfa.add_state(st);
                    stack.push(Fragment::detached(idx));
                }
                Expr::CharRange(l, r) => {
                    let chars = Self::range_chars(l, r)?;
                    let st = State::Transition {
                        id: counter,
                        condition: Condition::CharClass(chars),
                        output: None,
                    };
                    let idx = nfa.add_state(st);
                    stack.push(Fragment::detached(idx));
                }
                Expr::Concat => {
                    let right = stack.pop().ok_or("Missing right fragment")?;
                    let mut left = stack.pop().ok_or("Missing left fragment")?;
                    nfa.link_fragments(&mut left, right)?;
                    stack.push(left);
                }
                Expr::Alt => {
                    let right = stack.pop().ok_or("Missing right fragment")?;
                    let left = stack.pop().ok_or("Missing left fragment")?;
                    let split = State::Split {
                        id: counter,
                        left: Some(left.head),
                        right: Some(right.head),
                    };
                    let idx = nfa.add_state(split);
                    if stack.is_empty() {
                        nfa.head = idx;
                    }
                    let merged = Fragment::multi_link(idx, left.out, right.out);
                    stack.push(merged);
                }
                Expr::Opt => {
                    let e = stack.pop().ok_or("Missing fragment for '?' operator")?;
                    let split = State::Split {
                        id: counter,
                        left: Some(e.head),
                        right: None,
                    };
                    let idx = nfa.add_state(split);
                    nfa.head = idx;
                    let new_frag = Fragment::multi_link(idx, e.out, vec![idx]);
                    stack.push(new_frag);
                }
                Expr::Star => {
                    let mut e = stack.pop().ok_or("Missing fragment for '*' operator")?;

                    let split = State::Split {
                        id: counter,
                        left: Some(e.head),
                        right: None,
                    };
                    let idx = nfa.add_state(split.clone());
                    nfa.link_fragment(&mut e, idx)?;
                    if stack.is_empty() {
                        nfa.head = idx;
                    }
                    stack.push(Fragment::detached(idx));
                }
                Expr::Plus => {
                    let mut e = stack.pop().ok_or("Missing fragment for '+' operator")?;
                    let split = State::Split {
                        id: counter,
                        left: Some(e.head),
                        right: None,
                    };
                    let idx = nfa.add_state(split.clone());
                    nfa.link_fragment(&mut e, idx)?;
                    let new_frag = Fragment::single_link(e.head, idx);
                    stack.push(new_frag);
                }
            }
            counter += 1;
        }

        let mut final_fragment = stack.pop().ok_or("No final fragment on stack")?;
        let accept_idx = nfa.add_state(State::Accept { id: counter });
        nfa.link_fragments(&mut final_fragment, Fragment::detached(accept_idx))?;
        Ok(nfa)
    }

    pub fn to_string(&self) -> String {
        let mut s = format!("head = {}\n", self.head);
        for (i, st) in self.state_list.iter().enumerate() {
            s.push_str(&format!("(idx = {i} {})\n", st.to_string()));
        }
        s
    }

    pub fn print(&self) {
        println!("{}", self.to_string());
    }

    pub fn start(&self) -> usize {
        self.head
    }

    pub fn get_state(&self, idx: usize) -> State {
        self.state_list[idx].clone()
    }

    pub fn size(&self) -> usize {
        self.state_list.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{Expr, NFA};

    fn run_test(input: &str, expected: &str) {
        let expr = Expr::build(input).unwrap();
        let nfa = NFA::build(expr).expect("Failed to build NFA");
        let actual_str = nfa.to_string();
        let actual = actual_str.trim();
        assert_eq!(actual, expected.trim(), "Mismatch for input: {}", input);
    }

    #[test]
    fn test_simple_expression() {
        run_test(
            "a.b",
            r#"
head = 0
(idx = 0 [match 'a' -> Some(1)])
(idx = 1 [match 'b' -> Some(2)])
(idx = 2 [accept])
"#,
        );
    }

    #[test]
    fn test_alternation() {
        run_test(
            "a|b",
            r#"
head = 2
(idx = 0 [match 'a' -> Some(3)])
(idx = 1 [match 'b' -> Some(3)])
(idx = 2 [-> (Some(0) | Some(1))])
(idx = 3 [accept])
"#,
        );
    }

    #[test]
    fn test_nested_alternation_and_concat() {
        run_test(
            "(a.b)|(c|d)",
            r#"
head = 5
(idx = 0 [match 'a' -> Some(1)])
(idx = 1 [match 'b' -> Some(6)])
(idx = 2 [match 'c' -> Some(6)])
(idx = 3 [match 'd' -> Some(6)])
(idx = 4 [-> (Some(2) | Some(3))])
(idx = 5 [-> (Some(0) | Some(4))])
(idx = 6 [accept])
"#,
        );
    }

    #[test]
    fn test_kleene_star() {
        run_test(
            "a*",
            r#"
head = 1
(idx = 0 [match 'a' -> Some(1)])
(idx = 1 [-> (Some(0) | Some(2))])
(idx = 2 [accept])
"#,
        );
    }

    #[test]
    fn test_plus_operator() {
        run_test(
            "a+",
            r#"
head = 0
(idx = 0 [match 'a' -> Some(1)])
(idx = 1 [-> (Some(0) | Some(2))])
(idx = 2 [accept])
"#,
        );
    }

    #[test]
    fn test_optional_operator() {
        run_test(
            "a?",
            r#"
head = 1
(idx = 0 [match 'a' -> Some(2)])
(idx = 1 [-> (Some(0) | Some(2))])
(idx = 2 [accept])
"#,
        );
    }

    #[test]
    fn test_complex_expression() {
        run_test(
            "a.b.c|d*",
            r#"
head = 5
(idx = 0 [match 'a' -> Some(1)])
(idx = 1 [match 'b' -> Some(2)])
(idx = 2 [match 'c' -> Some(6)])
(idx = 3 [match 'd' -> Some(4)])
(idx = 4 [-> (Some(3) | Some(6))])
(idx = 5 [-> (Some(0) | Some(4))])
(idx = 6 [accept])
"#,
        );
    }
}
