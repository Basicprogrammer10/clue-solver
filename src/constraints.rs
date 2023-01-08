use std::{fmt::Display, ops::Deref};

use crate::element::{ElementIdentifier, ElementState, ElementType, Elements, ProcesResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// A token garented to be a tree
pub struct Constraint(Token);

pub enum Solvable {
    Yes(ElementIdentifier),
    AlreadySolved,
    No,
}

impl Constraint {
    // Parse boolean expressions like:
    // w1 | l3 | p5
    pub fn parse(raw: &str) -> Result<Self, ProcesResult> {
        let tokens = tokenize::tokenize(raw)?;
        let tree = tree::parse(tokens)?;
        Ok(tree)
    }

    // Walk the tree and make sure no more than 1 element is unknown
    // if returns None then there is more than 1 unknown element
    // if returns Some then there is 1 unknown element
    pub fn solveable(&self, elements: &Elements) -> Solvable {
        let mut missing = None;

        for i in self.flatten_tree() {
            if let Token::Element(id) = i {
                if elements.get_state(id) == ElementState::Unknown {
                    if missing.is_some() {
                        return Solvable::No;
                    }
                    missing = Some(id);
                }
            }
        }

        match missing {
            Some(x) => Solvable::Yes(*x),
            None => Solvable::AlreadySolved,
        }
    }

    // Return the element it has info for and its solved state
    pub fn solve(&self, elements: &Elements) -> Result<(ElementIdentifier, SolvedState), Solvable> {
        let solve_for = match self.solveable(elements) {
            Solvable::Yes(x) => x,
            x => return Err(x),
        };

        const TEST_STATES: [ElementState; 2] = [ElementState::Confirmed, ElementState::Dismissed];
        let mut result = [false; 2];

        for i in 0..TEST_STATES.len() {
            let mut elements = elements.clone();
            elements.set_state(&solve_for, TEST_STATES[i]);
            result[i] = self._evaluate(&self.0, &elements);
        }

        Ok((
            solve_for,
            match result {
                [true, false] => SolvedState::Confirmed,
                [false, true] => SolvedState::Dismissed,
                [true, true] => SolvedState::Any,
                _ => unreachable!(),
            },
        ))
    }

    // true -> Confirmed
    // false -> Dismissed
    fn _evaluate(&self, token: &Token, elements: &Elements) -> bool {
        match token {
            Token::Tree(op, a, b) => {
                let a = self._evaluate(a, elements);
                let b = self._evaluate(b, elements);

                match op {
                    Ops::Or => a || b,
                }
            }
            Token::Element(id) => {
                let state = elements.get_state(id);
                match state {
                    ElementState::Confirmed => true,
                    ElementState::Dismissed => false,
                    ElementState::Unknown => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum SolvedState {
    Confirmed,
    Dismissed,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Op(Ops),
    Element(ElementIdentifier),
    Tree(Ops, Box<Token>, Box<Token>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ops {
    Or,
}
mod tokenize {
    use super::*;

    struct TokenizeContext {
        out: Vec<Token>,
        working: String,
    }

    pub fn tokenize(raw: &str) -> Result<Vec<Token>, ProcesResult> {
        let mut ctx = TokenizeContext::new();

        for i in raw.chars() {
            match i {
                x if x.is_whitespace() => continue,
                '|' => ctx.operator(Ops::Or)?,
                x => ctx.working.push(x),
            }
        }

        ctx.flush()?;
        Ok(ctx.out)
    }

    impl TokenizeContext {
        fn new() -> Self {
            Self {
                out: Vec::new(),
                working: String::new(),
            }
        }

        fn flush(&mut self) -> Result<(), ProcesResult> {
            if self.working.is_empty() {
                return Ok(());
            }

            let mut chars = self.working.chars();
            let element_type = match chars.next() {
                Some('w') => ElementType::Weapon,
                Some('l') => ElementType::Location,
                Some('p') => ElementType::Person,
                _ => return Err(ProcesResult::InvalidSection),
            };

            let index = chars
                .take_while(|x| x.is_ascii_digit())
                .collect::<String>()
                .parse::<usize>()
                .ok()
                .map(|x| x.saturating_sub(1))
                .ok_or(ProcesResult::InvalidIndex)?;

            self.working.clear();
            self.out.push(Token::Element(ElementIdentifier {
                element_type,
                index,
            }));
            Ok(())
        }

        fn operator(&mut self, op: Ops) -> Result<(), ProcesResult> {
            self.flush()?;
            self.out.push(Token::Op(op));
            Ok(())
        }
    }

    impl Token {
        pub fn flatten_tree(&self) -> Vec<&Self> {
            match self {
                Token::Tree(_, left, right) => {
                    let mut out = left.flatten_tree();
                    out.extend(right.flatten_tree());
                    out
                }
                x => vec![x],
            }
        }
    }
}

mod tree {
    use super::*;

    pub fn parse(mut tokens: Vec<Token>) -> Result<Constraint, ProcesResult> {
        if tokens.len() <= 1 {
            return Err(ProcesResult::InvalidConstraint);
        }

        while tokens.len() > 1 {
            let mut i = 0;

            while i < tokens.len() {
                if let Token::Op(e) = tokens[i] {
                    let left = safe_remove(&mut tokens, i as isize - 1)?;
                    let right = safe_remove(&mut tokens, i as isize)?;

                    tokens[i - 1] = Token::Tree(e, Box::new(left), Box::new(right));
                    break;
                }

                i += 1;
            }
        }

        if tokens.len() != 1 || !matches!(tokens[0], Token::Tree(..)) {
            return Err(ProcesResult::InvalidConstraint);
        }

        Ok(Constraint(tokens[0].clone()))
    }

    fn safe_remove(tokens: &mut Vec<Token>, index: isize) -> Result<Token, ProcesResult> {
        if index < 0 || index as usize >= tokens.len() {
            return Err(ProcesResult::InvalidConstraint);
        }

        Ok(tokens.remove(index as usize))
    }
}

impl Deref for Constraint {
    type Target = Token;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Op(op) => write!(f, "{}", op),
            Token::Element(id) => write!(f, "{}", id),
            Token::Tree(op, left, right) => write!(f, "{} {} {}", left, op, right),
        }
    }
}

impl Display for Ops {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ops::Or => write!(f, "|"),
        }
    }
}
