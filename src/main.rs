use std::{io::stdin, sync::Arc};

use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, SetTitle},
};
use element::Elements;

mod app;
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
        println!("You typed: {}", line);
        ui::draw(app.clone());
    }
}
