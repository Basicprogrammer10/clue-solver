use std::{
    io::{stdout, Stdout},
    process,
};

use crossterm::{execute, terminal::LeaveAlternateScreen};
use hashbrown::{HashMap, HashSet};
use parking_lot::RwLock;

use crate::{
    constraints::{Constraint, Solvable, SolvedState},
    element::{ElementIdentifier, Elements, ProcesResult},
};

pub struct App {
    pub elements: RwLock<Elements>,
    pub constraints: RwLock<Vec<Constraint>>,
    pub constraint_cache: RwLock<HashMap<ElementIdentifier, Option<SolvedState>>>,

    // == UI ==

    // (command, error)
    pub unsolved_constraints: RwLock<HashSet<Constraint>>,
    pub command_history: RwLock<Vec<(String, Option<String>)>>,
    pub stdout: Stdout,
}

impl App {
    pub fn new(elements: Elements) -> Self {
        Self {
            elements: RwLock::new(elements),
            constraints: RwLock::new(Vec::new()),
            constraint_cache: RwLock::new(HashMap::new()),

            unsolved_constraints: RwLock::new(HashSet::new()),
            command_history: RwLock::new(Vec::new()),
            stdout: stdout(),
        }
    }

    pub fn refresh_constraints(&self) {
        let mut cache = self.constraint_cache.write();
        let mut unsolved = self.unsolved_constraints.write();
        let constraints = self.constraints.read();
        let elements = self.elements.read();
        unsolved.clear();
        cache.clear();

        for constraint in constraints.iter() {
            match constraint.solve(&*elements) {
                Ok((element, state)) => {
                    cache.insert(element, Some(state));
                }
                Err(Solvable::No | Solvable::AlreadySolved) => {
                    unsolved.insert(constraint.to_owned());
                }
                _ => {}
            }
        }
    }

    pub fn execute_command(&self, command: String) {
        if command == "exit" {
            execute!(self.stdout.lock(), LeaveAlternateScreen).unwrap();
            process::exit(0);
        }

        let mut elements = self.elements.write();
        let mut error = match elements.process_action(&command) {
            ProcesResult::Success => None,
            x => Some(x.to_string()),
        };

        if error.is_some() {
            match Constraint::parse(&command) {
                Ok(x) => {
                    self.constraints.write().push(x);
                    error = None;
                }
                _ => {}
            };
        }

        self.command_history.write().push((command, error));
    }
}
