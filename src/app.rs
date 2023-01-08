use std::{
    io::{stdout, Stdout},
    process,
};

use crossterm::{execute, terminal::LeaveAlternateScreen};
use parking_lot::RwLock;

use crate::{
    constraints::Constraint,
    element::{Elements, ProcesResult},
};

pub struct App {
    pub elements: RwLock<Elements>,
    pub constraints: RwLock<Vec<Constraint>>,
    // (command, error)
    pub command_history: RwLock<Vec<(String, Option<String>)>>,

    pub stdout: Stdout,
}

impl App {
    pub fn new(elements: Elements) -> Self {
        Self {
            elements: RwLock::new(elements),
            constraints: RwLock::new(Vec::new()),
            command_history: RwLock::new(Vec::new()),
            stdout: stdout(),
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
