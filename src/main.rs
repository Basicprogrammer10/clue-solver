use std::{io::stdin, sync::Arc};

use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, SetTitle},
};
use element::Elements;

mod app;
mod constraints;
mod element;
mod ui;

fn main() {
    let elements = Elements::load("./elements.toml").unwrap();
    let app = Arc::new(App::new(elements));
    execute!(
        app.stdout.lock(),
        EnterAlternateScreen,
        SetTitle("Clue Solver")
    )
    .unwrap();

    ui::draw(app.clone());
    for line in stdin().lines().map(Result::unwrap) {
        app.execute_command(line);
        app.refresh_constraints();
        ui::draw(app.clone());
    }
}
