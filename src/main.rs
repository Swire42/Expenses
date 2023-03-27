use std::io::{stdout, Write};
use chrono::{Local, DateTime, TimeZone, NaiveDate};

enum InputEvent {
    Up,
    Down,
    Esc,
    Char(char),
}

fn get_event() -> crossterm::Result<InputEvent> {
    use crossterm::event::{read, Event, KeyCode};

    loop {
        let event = read()?;

        if let Event::Key(key_event) = event {
            if key_event == KeyCode::Esc.into() {
                return Ok(InputEvent::Esc);
            } else if key_event == KeyCode::Up.into() {
                return Ok(InputEvent::Up);
            } else if key_event == KeyCode::Down.into() {
                return Ok(InputEvent::Down);
            } else if let KeyCode::Char(c) = key_event.code {
                return Ok(InputEvent::Char(c));
            }
        }
    }
}

fn app() -> crossterm::Result<()> {
    let mut date: NaiveDate = Local::now().date_naive();

    use crossterm::{
        terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
        execute,
        queue,
        cursor,
        style::Print,
    };

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;

    loop {
        queue!(stdout(), Clear(ClearType::All))?;
        queue!(stdout(), cursor::MoveTo(0, 0), Print("Press ESC to quit."))?;
        queue!(stdout(), cursor::MoveTo(0, 1), Print(format!("{}", date.format("%d-%m-%Y").to_string())))?;

        stdout().flush()?;

        use InputEvent::*;

        match get_event()? {
            Esc => break,
            Up => date = date.succ_opt().unwrap(),
            Down => date = date.pred_opt().unwrap(),
            _ => (),
        }
    }

    disable_raw_mode()?;
    execute!(stdout(), cursor::Show, LeaveAlternateScreen)?;

    Ok(())
}

fn main() {
    app().unwrap();
}
