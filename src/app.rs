use std::io::{stdout, Stdout};

use parking_lot::RwLock;

use crate::element::Elements;

type Constraint = ();

pub struct App {
    pub elements: RwLock<Elements>,
    pub constraints: Vec<Constraint>,
    pub stdout: Stdout,
}

impl App {
    pub fn new(elements: Elements) -> Self {
        Self {
            elements: RwLock::new(elements),
            constraints: Vec::new(),
            stdout: stdout(),
        }
    }
}
