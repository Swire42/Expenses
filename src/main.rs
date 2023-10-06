mod term;
mod interface;
mod tags;
mod accounts;
mod transaction;
mod money;
mod moneystate;
mod color;
mod datetime;
mod yamlrw;
mod completion;

use std::error::Error;
use crate::term::TermElement;

fn app() -> Result<(), Box<dyn Error>> {
    let mut app = interface::App::new();
    app.run()?;
    Ok(())
}

fn setup_panic_hook() {
    use crossterm::{
        terminal::{disable_raw_mode, LeaveAlternateScreen},
        execute,
        cursor,
    };
    use std::io::stdout;

    std::panic::set_hook(Box::new(|panic_info| {
        // Exits raw mode.
        disable_raw_mode().unwrap();
        execute!(stdout(), cursor::Show, cursor::SetCursorStyle::DefaultUserShape, LeaveAlternateScreen).unwrap();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
}

fn main() {
    setup_panic_hook();
    match app() {
        Err(err) => eprintln!("{err}"),
        _ => (),
    }
}
