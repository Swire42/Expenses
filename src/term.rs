use std::io::{stdout, Write};

#[derive(Copy, Clone, Debug)]
pub struct TermPos {
    pub col: usize,
    pub row: usize,
}

impl TermPos {
    pub fn new(col: usize, row: usize) -> Self {
        Self{col, row}
    }

    pub fn goto(&self) -> crossterm::Result<()> {
        use crossterm::{queue, cursor};
        queue!(stdout(), cursor::MoveTo(self.col.try_into().unwrap(), self.row.try_into().unwrap()))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TermBox {
    pub left: usize,
    pub right: usize,
    pub top: usize,
    pub bottom: usize,
}

impl TermBox {
    pub fn window() -> Self {
        let (cols, rows) = crossterm::terminal::size().unwrap();
        Self {
            left: 0,
            right: cols.into(),
            top: 0,
            bottom: rows.into(),
        }
    }

    pub fn begin(&self) -> TermPos {
        TermPos::new(self.left, self.top)
    }

    pub fn width(&self) -> usize {
        self.right - self.left
    }

    pub fn height(&self) -> usize {
        self.bottom - self.top
    }
}

pub fn simple_stylize<T: std::fmt::Display+crossterm::style::Stylize<Styled=crossterm::style::StyledContent<T>>>(text: T, color: crossterm::style::Color, bold: bool, reverse: bool) -> T::Styled {
    use crossterm::style::Stylize;

    let mut ret = text.stylize();

    if bold {
        ret = ret.bold();
    }

    if reverse {
        ret = ret.on(color);
        ret = ret.reverse();
    } else {
        ret = ret.with(color);
    }

    ret
}

pub fn subdiv_flex<const SIZE: usize>(total: usize, weights: [usize; SIZE]) -> [usize; SIZE] {
    assert!(SIZE > 0);
    let wsum: usize = weights.iter().sum();
    let mut ret = weights.map(|w| total * w / wsum);
    let ret_width: usize = ret.iter().sum();
    let rem_width = total - ret_width;
    for k in 0..rem_width {
        ret[k] += 1;
    }
    assert_eq!(ret.iter().sum::<usize>(), total);
    ret
}

pub fn subdiv_const_flex<const SIZE: usize>(total: usize, weights: [(usize, usize); SIZE]) -> [usize; SIZE] {
    assert!(SIZE > 0);
    let const_widths = weights.clone().map(|(c, _)| c);
    let const_width: usize = const_widths.iter().sum();
    assert!(total > const_width);
    let flex_width: usize = total - const_width;
    let flex_widths = subdiv_flex(flex_width, weights.map(|(_, f)| f));
    let mut ret = const_widths;
    for k in 0..SIZE {
        ret[k] += flex_widths[k];
    }
    assert_eq!(ret.iter().sum::<usize>(), total);
    ret
}

pub fn truncate_align_left(mut text: String, width: usize) -> String {
    text.truncate(width);
    format!("{: <width$}", text, width = width)
}

pub enum InputEvent {
    Up,
    Down,
    Left,
    Right,
    Esc,
    Backspace,
    Delete,
    Tab,
    BackTab,
    Enter,
    Char(char),
}

pub fn get_event() -> crossterm::Result<InputEvent> {
    use crossterm::event::{read, Event, KeyCode, KeyModifiers};

    loop {
        let event = read()?;

        if let Event::Key(key_event) = event {
            match key_event.modifiers {
                KeyModifiers::NONE => {
                    match key_event.code {
                        KeyCode::Esc => return Ok(InputEvent::Esc),
                        KeyCode::Enter => return Ok(InputEvent::Enter),
                        KeyCode::Up => return Ok(InputEvent::Up),
                        KeyCode::Down => return Ok(InputEvent::Down),
                        KeyCode::Left => return Ok(InputEvent::Left),
                        KeyCode::Right => return Ok(InputEvent::Right),
                        KeyCode::Backspace => return Ok(InputEvent::Backspace),
                        KeyCode::Delete => return Ok(InputEvent::Delete),
                        KeyCode::Tab => return Ok(InputEvent::Tab),
                        _ => ()
                    }
                },
                KeyModifiers::SHIFT => {
                    match key_event.code {
                        KeyCode::BackTab => return Ok(InputEvent::BackTab),
                        _ => ()
                    }
                },
                _ => (),
            }

            if let KeyCode::Char(c) = key_event.code {
                return Ok(InputEvent::Char(c));
            }
        }
    }
}

pub trait TermElement {
    fn display(&self, element_box: TermBox, active: bool) -> crossterm::Result<()>;

    fn popup(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()>;

    fn set_cursor(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()>;

    fn input(&mut self, event: InputEvent) -> Option<InputEvent>;

    fn run(&mut self) -> crossterm::Result<()> {
        use crossterm::{
            terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
            execute,
            cursor,
            queue,
        };

        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        loop {
            queue!(stdout(), Clear(ClearType::All))?;
            self.display(TermBox::window(), true)?;
            self.popup(TermBox::window(), TermBox::window())?;
            self.set_cursor(TermBox::window(), TermBox::window())?;
            stdout().flush()?;
            if self.input(get_event()?).is_some() {
                break;
            }
        }
        disable_raw_mode()?;
        execute!(stdout(), cursor::Show, cursor::SetCursorStyle::DefaultUserShape, LeaveAlternateScreen)?;

        Ok(())
    }
}
