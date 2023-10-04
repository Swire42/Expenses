mod term;
mod interface;
mod tags;
mod accounts;
mod transaction;
mod money;
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

fn main() {
    match app() {
        Err(err) => eprintln!("{err}"),
        _ => (),
    }
}
