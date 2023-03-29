use std::io::{stdout, Write};
use std::fmt;
use chrono::{Local, NaiveDate};

use crate::completion::Completor;

enum InputEvent {
    Up,
    Down,
    Left,
    Right,
    Esc,
    Backspace,
    Tab,
    BackTab,
    Enter,
    Char(char),
}

enum Action {
    Nothing,
    Next,
    Prev,
}

struct DateInput {
    date: NaiveDate,
}

impl DateInput {
    pub fn new(date: NaiveDate) -> Self {
        Self{date}
    }

    pub fn input(&mut self, event: InputEvent) -> Action {
        use InputEvent::*;
        match event {
            Tab | Enter => return Action::Next,
            BackTab => return Action::Prev,
            Down => self.date = self.date.succ_opt().unwrap(),
            Up => self.date = self.date.pred_opt().unwrap(),
            _ => (),
        }
        Action::Nothing
    }

    pub fn display(&self, line: u16, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            terminal,
            queue,
            cursor,
            style::{Print, PrintStyledContent, StyledContent, Stylize}
        };

        let mut tmp = self.date.format("%d-%m-%Y").to_string().bold();
        if active {
            tmp = tmp.reverse();
        }
        queue!(stdout(), cursor::MoveTo(0, line), PrintStyledContent(tmp))?;

        Ok(())
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

    pub fn input(&mut self, event: InputEvent) -> Action {
        use InputEvent::*;

        match event {
            Tab | Enter => return Action::Next,
            BackTab => return Action::Prev,
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
                Action::Nothing
            }
            Char(c) => {
                if let Some(Ok(val)) = c.to_digit(10).map(usize::try_from) {
                    match self.separator_dist {
                        None => {
                            self.cents = self.cents * 10 + val * 100;
                            Action::Nothing
                        },
                        Some(0) => {
                            self.cents += val * 10;
                            self.separator_dist = Some(1);
                            Action::Nothing
                        },
                        Some(1) => {
                            self.cents += val;
                            self.separator_dist = Some(2);
                            Action::Next
                        },
                        Some(2) => Action::Next,
                        _ => unreachable!(),
                    }
                } else if c == '.' || c == ',' {
                    if self.separator_dist == None {
                        self.separator_dist = Some(0);
                        Action::Nothing
                    } else {
                        Action::Next
                    }
                } else {
                    Action::Nothing
                }
            },
            _ => Action::Nothing,
        }
    }

    pub fn display(&self, line: u16, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            terminal,
            queue,
            cursor,
            style::{Print, PrintStyledContent, StyledContent, Stylize}
        };

        let mut tmp = format!("{self}").bold();
        if active {
            tmp = tmp.reverse();
        }
        queue!(stdout(), cursor::MoveTo(0, line), PrintStyledContent(tmp))?;

        Ok(())
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

struct TextInput {
    text: String,
}

impl TextInput {
    pub fn new() -> Self {
        Self{text: String::new()}
    }

    pub fn input(&mut self, event: InputEvent) -> Action {
        use InputEvent::*;

        match event {
            Tab | Enter => return Action::Next,
            BackTab => return Action::Prev,
            Backspace => {
                let _ = self.text.pop();
            },
            Char(c) => {
                self.text.push(c);
            },
            _ => (),
        }

        Action::Nothing
    }
}

impl fmt::Display for TextInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.text)
    }
}

pub struct CompletorInput {
    text: String,
    decor_prefix: char,
    decor_suffix: char,
    strict: bool, // enforce match
    compl: Completor,
    selection: Option<usize>,
}

impl CompletorInput {
    pub fn new(decor_prefix: char, decor_suffix: char, strict: bool, compl: Completor) -> Self {
        Self{text: String::new(), decor_prefix, decor_suffix, strict, compl, selection: None}
    }

    fn exit(&mut self) {
        if self.strict {
            self.text = self.compl.matches()[self.selection.unwrap_or(0)].clone();
            self.compl.update(&self.text);
            self.selection = None;
        } else {
            if let Some(n) = self.selection {
                self.text = self.compl.matches()[n].clone();
                self.compl.update(&self.text);
                self.selection = None;
            }
        }
    }

    pub fn input(&mut self, event: InputEvent) -> Action {
        use InputEvent::*;

        match event {
            Backspace => {
                let _ = self.text.pop();
                self.compl.update(&self.text);
                self.selection = None;
            },
            Char(c) => {
                self.text.push(c);
                self.compl.update(&self.text);
                self.selection = None;
                if self.strict && self.compl.matches().is_empty() {
                    let _ = self.text.pop();
                    self.compl.update(&self.text);
                }
            },
            Down => {
                if !self.compl.matches().is_empty() {
                    self.selection = Some(self.selection.map_or(0, |x| usize::min(x+1, self.compl.matches().len()-1)));
                }
            },
            Up => {
                self.selection = match self.selection {
                    None | Some(0) => None,
                    Some(x) => Some(x-1),
                }
            },
            Tab | Enter => {
                self.exit();
                return Action::Next;
            },
            BackTab => {
                self.exit();
                return Action::Prev;
            },
            _ => (),
        }

        Action::Nothing
    }

    pub fn display(&self, line: u16, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            terminal,
            queue,
            cursor,
            style::{Print, PrintStyledContent, StyledContent, Stylize}
        };

        let mut tmp = format!("{}{}{}", self.decor_prefix, self.text, self.decor_suffix).bold();
        if active && self.selection.is_none() {
            tmp = tmp.reverse();
        }
        queue!(stdout(), cursor::MoveTo(0, line), PrintStyledContent(tmp))?;

        if active {
            let (cols, rows) = terminal::size()?;
            for (lig, (n, sugg)) in ((line+1)..rows).zip(self.compl.matches().iter().enumerate()) {
                let tmp = if Some(n) == self.selection {
                    format!(">{}<", sugg).bold().reverse()
                } else {
                    format!(" {} ", sugg).bold()
                };
                queue!(stdout(), cursor::MoveTo(0, lig), PrintStyledContent(tmp))?;
            }
            queue!(stdout(), cursor::MoveTo(1+self.text.chars().count() as u16, line), cursor::Show)?;
            if self.selection.is_none() {
                queue!(stdout(), cursor::SetCursorStyle::BlinkingBar)?;
            } else {
                queue!(stdout(), cursor::SetCursorStyle::SteadyBar)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PurchaseInputFocus {
    Date,
    Amount,
    Desc,
    Tag,
}

impl PurchaseInputFocus {
    pub fn new() -> Self {
        Self::Date
    }

    pub fn next(&mut self) {
        use PurchaseInputFocus::*;
        *self = match self {
            Date => Amount,
            Amount => Desc,
            Desc => Tag,
            Tag => Date,
        }
    }

    pub fn prev(&mut self) {
        use PurchaseInputFocus::*;
        *self = match self {
            Date => Tag,
            Amount => Date,
            Desc => Amount,
            Tag => Desc,
        }
    }
}

struct PurchaseInput {
    focus: PurchaseInputFocus,
    date: DateInput,
    amount: AmountInput,
    desc: CompletorInput,
    tag: CompletorInput,
}

impl PurchaseInput {
    pub fn new(date: NaiveDate, desc_completor: Completor, tags_completor: Completor) -> Self {
        Self{
            focus: PurchaseInputFocus::new(),
            date: DateInput::new(date),
            amount: AmountInput::new(),
            desc: CompletorInput::new('"', '"', false, desc_completor),
            tag: CompletorInput::new('<', '>', true, tags_completor),
        }
    }

    pub fn input(&mut self, event: InputEvent) {
        use PurchaseInputFocus::*;

        let action = match self.focus {
            Date => self.date.input(event),
            Amount => self.amount.input(event),
            Desc => self.desc.input(event),
            Tag => self.tag.input(event),
        };

        match action {
            Action::Nothing => (),
            Action::Next => self.focus.next(),
            Action::Prev => self.focus.prev(),
        }
    }

    pub fn display(&mut self, line: u16) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            cursor,
            style::{Print, PrintStyledContent, StyledContent}
        };

        fn apply_style(text: String, selected: bool) -> StyledContent<String> {
            use crossterm::style::Stylize;

            if selected {
                text.bold().reverse()
            } else {
                text.bold()
            }
        }

        use PurchaseInputFocus::*;
        let focus = self.focus;

        if focus != Date {self.date.display(line, false)?}
        if focus != Amount {self.amount.display(line+1, false)?}
        if focus != Desc {self.desc.display(line+2, false)?}
        if focus != Tag {self.tag.display(line+3, false)?}

        if focus == Date {self.date.display(line, true)?}
        if focus == Amount {self.amount.display(line+1, true)?}
        if focus == Desc {self.desc.display(line+2, true)?}
        if focus == Tag {self.tag.display(line+3, true)?}

        Ok(())
    }
}

fn get_event() -> crossterm::Result<InputEvent> {
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

pub fn app() -> crossterm::Result<()> {
    let today: NaiveDate = Local::now().date_naive();

    use crate::tags;
    use crate::yamlrw::YamlRW;
    let mut tags = tags::Tags::read_yaml("tags.yaml").unwrap();
    tags.fix();
    let desc_completor = Completor::new(Vec::new());
    let tags_completor = Completor::new(tags.0.into_keys().collect());

    let mut transaction = PurchaseInput::new(today, desc_completor, tags_completor);

    use crossterm::{
        terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
        execute,
        queue,
        cursor,
        style::{Print}
    };

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;

    loop {
        queue!(stdout(), Clear(ClearType::All), cursor::Hide)?;
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
