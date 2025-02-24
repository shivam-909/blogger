use super::{
    expr::Expr,
    nfa::{State, NFA},
};

use std::{
    collections::{HashMap, HashSet},
    sync::Mutex,
};

pub trait Match: Sync {
    fn matches(&self, s: &str) -> bool;
}

pub struct Matcher {
    pub nfa: NFA,
    epsilon_closure_cache: Mutex<HashMap<usize, Vec<State>>>,
}

impl Matcher {
    pub fn new(s: &str) -> Result<Self, String> {
        let expr = Expr::build(s)?;
        let nfa = NFA::build(expr)?;
        let epsilon_closure_cache = Self::precompute_epsilon_closures(&nfa);
        Ok(Self {
            nfa,
            epsilon_closure_cache: Mutex::new(epsilon_closure_cache),
        })
    }

    fn precompute_epsilon_closures(nfa: &NFA) -> HashMap<usize, Vec<State>> {
        (0..nfa.size())
            .map(|idx| {
                let mut seen = HashSet::new();
                (
                    idx,
                    Self::compute_epsilon_closure(nfa, &mut seen, &nfa.get_state(idx)),
                )
            })
            .collect()
    }

    fn compute_epsilon_closure(nfa: &NFA, seen: &mut HashSet<usize>, state: &State) -> Vec<State> {
        if !seen.insert(state.get_id()) {
            return Vec::new();
        }
        match state {
            State::Split { left, right, .. } => {
                let mut out = vec![state.clone()];
                out.extend(
                    left.map(|idx| Self::compute_epsilon_closure(nfa, seen, &nfa.get_state(idx)))
                        .unwrap_or_default(),
                );
                out.extend(
                    right
                        .map(|idx| Self::compute_epsilon_closure(nfa, seen, &nfa.get_state(idx)))
                        .unwrap_or_default(),
                );
                out
            }
            _ => vec![state.clone()],
        }
    }

    pub fn matches(&self, s: &str) -> bool {
        let ecc = self.epsilon_closure_cache.lock().unwrap();
        let start = ecc.get(&self.nfa.start()).cloned().unwrap_or_default();
        let final_states = s.chars().fold(start, |current, c| {
            current
                .into_iter()
                .flat_map(|st| match st {
                    State::Transition { output, .. } if st.matches_condition(c) => output
                        .and_then(|o| ecc.get(&o))
                        .cloned()
                        .unwrap_or_default(),
                    _ => Vec::new(),
                })
                .collect()
        });
        final_states
            .iter()
            .any(|st| matches!(st, State::Accept { .. }))
    }
}

impl Match for Matcher {
    fn matches(&self, s: &str) -> bool {
        self.matches(s)
    }
}

#[cfg(test)]
mod tests {
    use super::Matcher;

    #[test]
    fn test_simple_literal_match() {
        let matcher = Matcher::new("a").expect("Failed to build Matcher");
        assert!(matcher.matches("a"));
        assert!(!matcher.matches("b"));
    }

    #[test]
    fn test_concat_match() {
        let matcher = Matcher::new("a.b").expect("Failed to build Matcher");
        assert!(matcher.matches("ab"));
        assert!(!matcher.matches("a"));
        assert!(!matcher.matches("abc"));
    }

    #[test]
    fn test_alternation_match() {
        let matcher = Matcher::new("a|b").expect("Failed to build Matcher");
        assert!(matcher.matches("a"));
        assert!(matcher.matches("b"));
        assert!(!matcher.matches("c"));
    }

    #[test]
    fn test_kleene_star_match() {
        let matcher = Matcher::new("a*").expect("Failed to build Matcher");
        assert!(matcher.matches(""));
        assert!(matcher.matches("a"));
        assert!(matcher.matches("aaa"));
        assert!(!matcher.matches("b"));
    }

    #[test]
    fn test_plus_operator_match() {
        let matcher = Matcher::new("a+").expect("Failed to build Matcher");
        assert!(!matcher.matches(""));
        assert!(matcher.matches("a"));
        assert!(matcher.matches("aaa"));
        assert!(!matcher.matches("b"));
    }

    #[test]
    fn test_optional_operator_match() {
        let matcher = Matcher::new("a?").expect("Failed to build Matcher");
        assert!(matcher.matches(""));
        assert!(matcher.matches("a"));
        assert!(!matcher.matches("aa"));
        assert!(!matcher.matches("b"));
    }

    #[test]
    fn test_complex_expression_match() {
        let matcher = Matcher::new("(a.b)|(c*)").expect("Failed to build Matcher");
        assert!(matcher.matches("ab"));
        assert!(matcher.matches("")); // Matches "c*" with zero occurrences
        assert!(matcher.matches("ccc"));
        assert!(!matcher.matches("ac"));
        assert!(!matcher.matches("d"));
    }

    #[test]
    fn test_nested_alternation_and_concat() {
        let matcher = Matcher::new("(a.b)|(c|d)").expect("Failed to build Matcher");
        assert!(matcher.matches("ab"));
        assert!(matcher.matches("c"));
        assert!(matcher.matches("d"));
        assert!(!matcher.matches("a"));
        assert!(!matcher.matches("b"));
    }

    #[test]
    fn test_no_match() {
        let matcher = Matcher::new("a.b").expect("Failed to build Matcher");
        assert!(!matcher.matches(""));
        assert!(!matcher.matches("a"));
        assert!(!matcher.matches("b"));
    }

    #[test]
    fn test_repetition_and_alternation() {
        let matcher = Matcher::new("(a|b)*").expect("Failed to build Matcher");
        assert!(matcher.matches(""));
        assert!(matcher.matches("a"));
        assert!(matcher.matches("b"));
        assert!(matcher.matches("abab"));
        assert!(!matcher.matches("c"));
    }

    #[test]
    fn test_complex_repetition() {
        let matcher = Matcher::new("(a.b)*").expect("Failed to build Matcher");
        assert!(matcher.matches(""));
        assert!(matcher.matches("ab"));
        assert!(matcher.matches("abab"));
        assert!(!matcher.matches("a"));
        assert!(!matcher.matches("abc"));
    }
}
