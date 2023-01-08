use std::{io::Write, sync::Arc};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::Print,
    terminal::{Clear, ClearType},
    QueueableCommand,
};

use crate::{app::App, element::ElementState};

type Lines = Vec<String>;

pub fn draw(app: Arc<App>) {
    let mut stdout = app.stdout.lock();
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0)).unwrap();

    let columns = [elements::get(app.clone()), console::get(app.clone())];
    let max_len = columns.iter().map(|x| x.len()).max().unwrap_or(0);

    for line in 0..max_len {
        for column in &columns {
            match column.get(line) {
                Some(line) => queue!(stdout, Print(line), Print(" ")).unwrap(),
                None => queue!(stdout, Print(" ".repeat(max_len + 1))).unwrap(),
            }
        }
        queue!(stdout, Print("\n")).unwrap();
    }
    stdout
        .queue(MoveTo(columns[0][0].len() as u16 + 5, 1))
        .unwrap();

    app.stdout.lock().flush().unwrap();
}

mod elements {
    use super::*;

    pub fn get(app: Arc<App>) -> Lines {
        let max_name_length = app.elements.read().max_name_length;
        get_draw(app)
            .into_iter()
            .map(|element| match element {
                Draw::Separator(title) => {
                    let padding = "-".repeat(max_name_length + 1 - title.len());
                    format!("+-+-{}{}+", title, padding)
                }
                Draw::Element(name, state) => format!(
                    "|{}| {}{} |",
                    state.as_char(),
                    name,
                    " ".repeat(max_name_length - name.len())
                ),
            })
            .collect()
    }

    fn get_draw(app: Arc<App>) -> Vec<Draw> {
        let mut out = Vec::new();
        let card = app.elements.read();

        // Todo: Replace this with iterator magic
        out.push(Draw::Separator("(L)ocations".to_string()));
        for element in &card.locations {
            out.push(Draw::Element(element.name.clone(), element.state));
        }
        out.push(Draw::Separator("(P)eople".to_string()));
        for element in &card.people {
            out.push(Draw::Element(element.name.clone(), element.state));
        }
        out.push(Draw::Separator("(W)eapons".to_string()));
        for element in &card.weapons {
            out.push(Draw::Element(element.name.clone(), element.state));
        }
        out.push(Draw::Separator("".to_string()));

        out
    }

    enum Draw {
        Separator(String),
        Element(String, ElementState),
    }
}

mod console {
    use super::*;

    pub fn get(app: Arc<App>) -> Lines {
        let mut lines = app
            .command_history
            .read()
            .iter()
            .rev()
            .take(5)
            .map(|x| {
                format!(
                    "{}: {}",
                    x.0,
                    match &x.1 {
                        Some(x) => x.as_str(),
                        None => "ok",
                    }
                )
            })
            .collect::<Vec<_>>();
        let max_len = lines.iter().map(|x| x.len()).max().unwrap_or(0).max(20);
        lines
            .iter_mut()
            .for_each(|x| *x = format!("| {}{} |", x, " ".repeat(max_len - x.len())));

        lines.insert(0, format!("| >{}|", " ".repeat(max_len)));
        lines.insert(0, format!("+-Console{}+", "-".repeat(max_len - 6)));
        lines.push(format!("+{}+", "-".repeat(max_len + 2)));

        lines.push(String::new());
        lines.extend(constraints::get(app));
        lines
    }
}

mod constraints {
    use super::*;

    pub fn get(app: Arc<App>) -> Lines {
        let mut lines = app
            .constraints
            .read()
            .iter()
            .rev()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();

        let max_len = lines.iter().map(|x| x.len()).max().unwrap_or(20);
        lines
            .iter_mut()
            .for_each(|x| *x = format!("| {}{} |", x, " ".repeat(max_len - x.len())));

        lines.insert(0, format!("+-Constraints{}+", "-".repeat(max_len - 10)));
        lines.push(format!("+{}+", "-".repeat(max_len + 2)));

        lines
    }
}
