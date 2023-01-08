use std::{
    io::{stdout, Stdout},
    process,
};

use crossterm::{execute, terminal::LeaveAlternateScreen};
use parking_lot::RwLock;

use crate::element::{Elements, ProcesResult};

type Constraint = ();

pub struct App {
    pub elements: RwLock<Elements>,
    pub constraints: Vec<Constraint>,
    // (command, error)
    pub command_history: RwLock<Vec<(String, Option<String>)>>,

    pub stdout: Stdout,
}

impl App {
    pub fn new(elements: Elements) -> Self {
        Self {
            elements: RwLock::new(elements),
            constraints: Vec::new(),
            command_history: RwLock::new(Vec::new()),
            stdout: stdout(),
        }
    }

    pub fn execute_command(&self, command: String) {
        if command == "exit" {
            execute!(self.stdout.lock(), LeaveAlternateScreen).unwrap();
            process::exit(0);
        }

        let error = match self.elements.write().process_action(&command) {
            ProcesResult::Success => None,
            x => Some(x.to_string()),
        };

        self.command_history.write().push((command, error));
    }
}
