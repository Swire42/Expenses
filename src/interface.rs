use std::io::{stdout, Write};
use std::fmt;
use chrono::{Local, NaiveDate};

enum InputEvent {
    Up,
    Down,
    Left,
    Right,
    Esc,
    Backspace,
    Tab,
    Char(char),
}

struct DateInput {
    date: NaiveDate,
}

impl DateInput {
    pub fn new(date: NaiveDate) -> Self {
        Self{date}
    }

    /// Return true if finised
    pub fn input(&mut self, event: InputEvent) -> bool {
        use InputEvent::*;
        match event {
            Up => self.date = self.date.succ_opt().unwrap(),
            Down => self.date = self.date.pred_opt().unwrap(),
            _ => (),
        }
        false
    }
}

impl fmt::Display for DateInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.date.format("%d-%m-%Y"))
    }
}

struct AmountInput {
    cents: usize,
    separator_dist: Option<usize>,
}

impl AmountInput {
    pub fn new() -> Self {
        Self {
            cents: 0,
            separator_dist: None,
        }
    }

    /// Return true if finised
    pub fn input(&mut self, event: InputEvent) -> bool {
        use InputEvent::*;

        match event {
            Backspace => {
                match self.separator_dist {
                    None => {
                        if self.cents < 100_000_000 {
                            self.cents = self.cents / 1000 * 100;
                        }
                    },
                    Some(0) => {
                        self.separator_dist = None;
                    },
                    Some(1) => {
                        self.cents = self.cents / 100 * 100;
                        self.separator_dist = Some(0);
                    },
                    Some(2) => {
                        self.cents = self.cents / 10 * 10;
                        self.separator_dist = Some(1);
                    },
                    _ => unreachable!(),
                }
                false
            }
            Char(c) => {
                if let Some(Ok(val)) = c.to_digit(10).map(usize::try_from) {
                    match self.separator_dist {
                        None => {
                            self.cents = self.cents * 10 + val * 100;
                            false
                        },
                        Some(0) => {
                            self.cents += val * 10;
                            self.separator_dist = Some(1);
                            false
                        },
                        Some(1) => {
                            self.cents += val;
                            self.separator_dist = Some(2);
                            true
                        },
                        Some(2) => true,
                        _ => unreachable!(),
                    }
                } else if c == '.' || c == ',' {
                    if self.separator_dist == None {
                        self.separator_dist = Some(0);
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            },
            _ => false,
        }
    }
}

impl fmt::Display for AmountInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cents / 100)?;
        match self.separator_dist {
            None => (),
            Some(0) => write!(f, ".")?,
            Some(1) => write!(f, ".{}", (self.cents / 10) % 10)?,
            Some(2) => write!(f, ".{:02}", self.cents % 100)?,
            _ => unreachable!(),
        }
        write!(f, " €")
    }
}

enum TISelector {
    Date,
    Amount,
}

impl TISelector {
    pub fn new() -> Self {
        Self::Date
    }

    pub fn next(&mut self) {
        use TISelector::*;
        *self = match self {
            Date => Amount,
            Amount => Date,
        }
    }
}

struct TransactionInput {
    selector: TISelector,
    date: DateInput,
    amount: AmountInput,
}

impl TransactionInput {
    pub fn new(date: NaiveDate) -> Self {
        Self{selector: TISelector::new(), date: DateInput::new(date), amount: AmountInput::new()}
    }

    pub fn input(&mut self, event: InputEvent) {
        use InputEvent::*;
        match event {
            Tab => self.selector.next(),
            _ => {
                use TISelector::*;
                let next = match self.selector {
                    Date => self.date.input(event),
                    Amount => self.amount.input(event),
                };

                if next {
                    self.selector.next();
                }
            }
        }
    }

    pub fn display(&mut self, line: u16) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            cursor,
            style::{Print, PrintStyledContent, Stylize}
        };

        queue!(stdout(), cursor::MoveTo(0, line))?;
        let tmp = self.date.to_string().bold();
        queue!(stdout(), PrintStyledContent(match self.selector { TISelector::Date => tmp.reverse(), _ => tmp, }))?;
        queue!(stdout(), Print("   "))?;
        let tmp = self.amount.to_string().bold();
        queue!(stdout(), PrintStyledContent(match self.selector { TISelector::Amount => tmp.reverse(), _ => tmp, }))?;

        Ok(())
    }
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
            } else if key_event == KeyCode::Left.into() {
                return Ok(InputEvent::Left);
            } else if key_event == KeyCode::Right.into() {
                return Ok(InputEvent::Right);
            } else if key_event == KeyCode::Backspace.into() {
                return Ok(InputEvent::Backspace);
            } else if key_event == KeyCode::Tab.into() {
                return Ok(InputEvent::Tab);
            } else if let KeyCode::Char(c) = key_event.code {
                return Ok(InputEvent::Char(c));
            }
        }
    }
}

pub fn app() -> crossterm::Result<()> {
    let today: NaiveDate = Local::now().date_naive();

    let mut transaction = TransactionInput::new(today);

    use crossterm::{
        terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
        execute,
        queue,
        cursor,
        style::{Print, PrintStyledContent, Stylize}
    };

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;

    loop {
        queue!(stdout(), Clear(ClearType::All))?;
        queue!(stdout(), cursor::MoveTo(0, 0), Print("Press ESC to quit."))?;
        transaction.display(2)?;

        stdout().flush()?;

        use InputEvent::*;

        match get_event()? {
            Esc => break,
            e => {
                transaction.input(e);
            },
        }
    }

    disable_raw_mode()?;
    execute!(stdout(), cursor::Show, LeaveAlternateScreen)?;

    Ok(())
}
