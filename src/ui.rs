use std::{
    fmt::Display,
    io::{StdoutLock, Write},
    sync::Arc,
};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{ContentStyle, Print, PrintStyledContent, StyledContent},
    terminal::{Clear, ClearType},
    QueueableCommand,
};

use crate::{app::App, element::ElementState};

type Lines = Vec<Line>;

pub fn draw(app: Arc<App>) {
    let mut stdout = app.stdout.lock();
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0)).unwrap();

    let columns = [elements::get(app.clone()), console::get(app.clone())];
    let max_len = columns.iter().map(|x| x.len()).max().unwrap_or(0);

    for line in 0..max_len {
        for column in &columns {
            match column.get(line) {
                Some(line) => {
                    line.queue(&mut stdout);
                    stdout.queue(Print(" ")).unwrap();
                }
                None => queue!(stdout, Print(" ".repeat(max_len + 1))).unwrap(),
            }
        }
        queue!(stdout, Print("\n")).unwrap();
    }
    stdout
        .queue(MoveTo(columns[0][0].len as u16 + 5, 1))
        .unwrap();

    app.stdout.lock().flush().unwrap();
}

mod elements {
    use crossterm::style::{Color, Stylize};
    use hashbrown::HashMap;

    use crate::{
        constraints::SolvedState,
        element::{ElementIdentifier, ElementType},
    };

    use super::*;

    pub fn get(app: Arc<App>) -> Lines {
        let app_cache = app.clone();
        let max_name_length = app.elements.read().max_name_length;
        let constraints = app_cache.constraint_cache.read();
        get_draw(app)
            .into_iter()
            .map(|element| match element {
                Draw::Separator(title) => {
                    let padding = "-".repeat(max_name_length + 1 - title.len());
                    format!("+-+-{}{}+", title, padding).into()
                }
                Draw::Element(id, name, state) => {
                    let len = name.len();
                    Line::from("|")
                        .append(state.as_char())
                        .styled(ContentStyle::new().with(match state {
                            ElementState::Confirmed => Color::Green,
                            ElementState::Dismissed => Color::Red,
                            _ => Color::Reset,
                        }))
                        .append("| ")
                        .append(name)
                        .styled(ContentStyle::new().with(get_element_color(id, &*constraints)))
                        .append(" ".repeat(max_name_length - len))
                        .append(" |")
                }
            })
            .collect()
    }

    fn get_element_color(
        id: ElementIdentifier,
        constraints: &HashMap<ElementIdentifier, Option<SolvedState>>,
    ) -> Color {
        match constraints.get(&id) {
            Some(Some(SolvedState::Confirmed)) => Color::Green,
            Some(Some(SolvedState::Dismissed)) => Color::Red,
            _ => Color::Reset,
        }
    }

    fn get_draw(app: Arc<App>) -> Vec<Draw> {
        let mut out = Vec::new();
        let card = app.elements.read();

        // Todo: Replace this with iterator magic
        out.push(Draw::Separator("(L)ocations".to_string()));
        for (i, element) in card.locations.iter().enumerate() {
            out.push(Draw::Element(
                ElementIdentifier {
                    element_type: ElementType::Location,
                    index: i,
                },
                element.name.clone(),
                element.state,
            ));
        }
        out.push(Draw::Separator("(P)eople".to_string()));
        for (i, element) in card.people.iter().enumerate() {
            out.push(Draw::Element(
                ElementIdentifier {
                    element_type: ElementType::Person,
                    index: i,
                },
                element.name.clone(),
                element.state,
            ));
        }
        out.push(Draw::Separator("(W)eapons".to_string()));
        for (i, element) in card.weapons.iter().enumerate() {
            out.push(Draw::Element(
                ElementIdentifier {
                    element_type: ElementType::Weapon,
                    index: i,
                },
                element.name.clone(),
                element.state,
            ));
        }
        out.push(Draw::Separator("".to_string()));

        out
    }

    enum Draw {
        Separator(String),
        Element(ElementIdentifier, String, ElementState),
    }
}

mod console {
    use crossterm::style::{Attribute, Color, Stylize};

    use super::*;

    pub fn get(app: Arc<App>) -> Lines {
        let mut lines = app
            .command_history
            .read()
            .iter()
            .rev()
            .take(3)
            .map(|x| {
                Line::from(format!("{}: ", x.0))
                    .append(x.1.clone().unwrap_or_else(|| "ok".to_owned()))
                    .styled(
                        ContentStyle::new()
                            .attribute(Attribute::Bold)
                            .with(match x.1 {
                                Some(_) => Color::Red,
                                None => Color::Green,
                            }),
                    )
            })
            .collect::<Vec<Line>>();

        if app.command_history.read().len() > 3 {
            lines.push(Line::from("...").styled(ContentStyle::new().with(Color::DarkGrey)));
        }

        let max_len = lines.iter().map(|x| x.len).max().unwrap_or(0).max(20);
        lines.iter_mut().for_each(|x| {
            *x = Line::from("| ")
                .append_line(x)
                .append(" ".repeat(max_len - x.len))
                .append(" |")
        });

        lines.insert(0, format!("| >{}|", " ".repeat(max_len)).into());
        lines.insert(
            0,
            Line::from("+-")
                .append("Console")
                .styled(ContentStyle::new().attribute(Attribute::Bold))
                .append("-".repeat(max_len - 6))
                .append("+"),
        );
        lines.push(format!("+{}+", "-".repeat(max_len + 2)).into());

        lines.push(String::new().into());
        lines.extend(constraints::get(app));
        lines
    }
}

mod constraints {
    use crossterm::style::{Color, Stylize};

    use super::*;

    pub fn get(app: Arc<App>) -> Lines {
        let unsolved = app.unsolved_constraints.read();

        let mut lines = app
            .constraints
            .read()
            .iter()
            .rev()
            .map(|x| {
                Line::from(x.to_string()).styled(ContentStyle::new().with(
                    if unsolved.contains(x) {
                        Color::DarkGrey
                    } else {
                        Color::Reset
                    },
                ))
            })
            .collect::<Vec<Line>>();

        let max_len = lines.iter().map(|x| x.len).max().unwrap_or(0).max(20);
        lines.iter_mut().for_each(|x| {
            *x = Line::from("| ")
                .append_line(x)
                .append(" ".repeat(max_len - x.len))
                .append(" |")
        });

        lines.insert(
            0,
            format!("+-Constraints{}+", "-".repeat(max_len - 10)).into(),
        );
        lines.push(format!("+{}+", "-".repeat(max_len + 2)).into());

        lines
    }
}

pub struct Line {
    elements: Vec<StyledContent<String>>,
    len: usize,
}

impl Line {
    fn content(&self) -> String {
        self.elements
            .iter()
            .map(|x| x.content().to_string())
            .collect::<Vec<_>>()
            .join("")
    }

    fn append<T: Into<Line>>(self, other: T) -> Self {
        let new = other.into();
        Self {
            len: self.len + new.len,
            elements: [self.elements, new.elements].concat(),
        }
    }

    fn append_line(self, other: &Line) -> Self {
        Self {
            len: self.len + other.len,
            elements: [self.elements, other.elements.clone()].concat(),
        }
    }

    // Append a style to the last element
    fn styled(self, style: ContentStyle) -> Self {
        Self {
            len: self.len,
            elements: {
                let mut elements = self.elements;
                let last = elements.pop().unwrap();
                elements.push(StyledContent::new(style, last.content().to_string()));
                elements
            },
        }
    }

    fn queue(&self, stdout: &mut StdoutLock) {
        for element in self.elements.iter() {
            queue!(stdout, PrintStyledContent(element.clone())).unwrap();
        }
    }
}

impl<T: Display> From<T> for Line {
    fn from(inp: T) -> Self {
        let inp = inp.to_string();

        Self {
            len: inp.len(),
            elements: vec![StyledContent::new(ContentStyle::default(), inp)],
        }
    }
}
