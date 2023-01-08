use std::{io::Write, sync::Arc};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::Print,
    terminal::{Clear, ClearType},
    QueueableCommand,
};

use crate::{app::App, element::ElementState};

pub fn draw(app: Arc<App>) {
    let mut stdout = app.stdout.lock();
    let max_name_length = app.elements.read().max_name_length;
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0)).unwrap();

    let separator = format!("+-+{}+", "-".repeat(max_name_length + 2));
    for element in get_draw(app.clone()) {
        match element {
            Draw::Separator => stdout.queue(Print(&separator)).unwrap(),
            Draw::Element(name, state) => stdout
                .queue(Print(format!(
                    "|{}| {}{} |",
                    state.as_char(),
                    name,
                    " ".repeat(max_name_length - name.len())
                )))
                .unwrap(),
        };
        stdout.queue(Print("\n")).unwrap();
    }
    stdout.queue(Print("\n> ")).unwrap();

    drop(stdout);
    app.stdout.lock().flush().unwrap();
}

fn get_draw(app: Arc<App>) -> Vec<Draw> {
    let mut out = Vec::new();
    let card = app.elements.read();

    // Todo: Replace this with iterator magic
    out.push(Draw::Separator);
    for element in &card.locations {
        out.push(Draw::Element(element.name.clone(), element.state));
    }
    out.push(Draw::Separator);
    for element in &card.people {
        out.push(Draw::Element(element.name.clone(), element.state));
    }
    out.push(Draw::Separator);
    for element in &card.weapons {
        out.push(Draw::Element(element.name.clone(), element.state));
    }
    out.push(Draw::Separator);

    out
}

enum Draw {
    Separator,
    Element(String, ElementState),
}
