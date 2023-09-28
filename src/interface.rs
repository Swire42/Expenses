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
    Delete,
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

    fn input(&mut self, event: InputEvent) -> Action {
        use InputEvent::*;
        match event {
            Tab | Enter => return Action::Next,
            BackTab => return Action::Prev,
            Down | Right => self.date = self.date.succ_opt().unwrap(),
            Up | Left => self.date = self.date.pred_opt().unwrap(),
            _ => (),
        }
        Action::Nothing
    }

    pub fn display(&self, line: u16, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            cursor,
            style::{PrintStyledContent, Stylize}
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

    fn input(&mut self, event: InputEvent) -> Action {
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
            queue,
            cursor,
            style::{PrintStyledContent, Stylize}
        };

        let mut tmp = format!("{self}").bold();
        if active {
            tmp = tmp.reverse();
        }
        queue!(stdout(), cursor::MoveTo(0, line), PrintStyledContent(tmp))?;
        if active {
            queue!(stdout(), cursor::MoveLeft(2), cursor::Show, cursor::SetCursorStyle::BlinkingBar)?;
        }

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

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn display_len(&self) -> usize {
        2+self.text.len()
    }

    pub fn clear(&mut self) {
        self.text = String::new();
        self.compl.update(&self.text);
        self.selection = None;
    }

    pub fn get(&self) -> String {
        self.text.clone()
    }

    fn exit(&mut self) {
        if self.strict && !self.is_empty() {
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

    fn input(&mut self, event: InputEvent) -> Action {
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
            style::{PrintStyledContent, Stylize}
        };

        let mut tmp: crossterm::style::StyledContent<String> = format!("{}{}{}", self.decor_prefix, self.text, self.decor_suffix).bold();
        if active && self.selection.is_none() {
            tmp = tmp.reverse();
        }
        queue!(stdout(), cursor::MoveTo(0, line), PrintStyledContent(tmp))?;

        if active {
            let (_cols, rows) = terminal::size()?;
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

struct UsersInput {
    new_user: CompletorInput,
    users: Vec<String>,
    selection: Option<usize>,
}

impl UsersInput {
    pub fn new(compl: Completor) -> Self {
        Self{new_user: CompletorInput::new('[', ']', true, compl), users: Vec::new(), selection: None}
    }

    pub fn add_user(&mut self, user: String) {
        if !self.users.contains(&user) {
            self.users.push(user);
        }
    }

    pub fn del_user(&mut self) {
        if let Some(x) = &self.selection {
            self.users.remove(*x);
            self.selection =
            if self.users.is_empty() {
                None
            } else {
                Some(usize::min(*x, self.users.len()-1))
            };
        } else {
            panic!("No user selected");
        }
    }

    fn exit(&mut self) {
        if self.selection.is_none() {
            self.new_user.exit();
        } else {
            self.selection = None;
        }
    }

    fn validate_new_user(&mut self) {
        if !self.new_user.is_empty() {
            self.new_user.exit();
            self.add_user(self.new_user.get());
            self.new_user.clear();
        }
    }

    fn input(&mut self, event: InputEvent) -> Action {
        use InputEvent::*;

        match event {
            Left => {
                self.validate_new_user();
                self.selection = match self.selection {
                    None | Some(0) => None,
                    Some(x) => Some(x-1),
                };
            },
            Right => {
                self.validate_new_user();
                self.selection =
                if self.users.is_empty() {
                    None
                } else {
                    match self.selection {
                        None => Some(0),
                        Some(x) => Some(usize::min(x+1, self.users.len()-1)),
                    }
                };
            },
            event => {
                match &self.selection {
                    None => {
                        return match self.new_user.input(event) {
                            action @ (Action::Prev | Action::Next) => {
                                if self.new_user.is_empty() {
                                    action
                                } else {
                                    self.validate_new_user();
                                    Action::Nothing
                                }
                            },
                            action => action,
                        };
                    },
                    Some(x) => {
                        match event {
                            Backspace | Delete => {
                                self.del_user();
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
                    },
                }
            },
        }

        Action::Nothing
    }

    pub fn display(&self, line: u16, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            terminal,
            queue,
            cursor,
            style::{Print, PrintStyledContent, Stylize}
        };

        if self.selection.is_some() {
            self.new_user.display(line, false)?;
        }

        queue!(stdout(), cursor::MoveTo(self.new_user.display_len().try_into().unwrap(), line));

        for (n, user) in self.users.iter().enumerate() {
            let mut tmp: crossterm::style::StyledContent<String> = format!("{}", user).bold();
            if active && self.selection == Some(n) {
                tmp = tmp.reverse();
            }
            queue!(stdout(), Print(" "), PrintStyledContent(tmp))?;
        }

        if self.selection.is_none() {
            self.new_user.display(line, active)?;
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
    Buyer,
    Users,
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
            Tag =>Buyer,
            Buyer => Users,
            Users => Date,
        }
    }

    pub fn prev(&mut self) {
        use PurchaseInputFocus::*;
        *self = match self {
            Date => Users,
            Amount => Date,
            Desc => Amount,
            Tag => Desc,
            Buyer => Tag,
            Users => Buyer,
        }
    }
}

struct PurchaseInput {
    focus: PurchaseInputFocus,
    date: DateInput,
    amount: AmountInput,
    desc: CompletorInput,
    tag: CompletorInput,
    buyer: CompletorInput,
    users: UsersInput,
}

impl PurchaseInput {
    pub fn new(date: NaiveDate, desc_completor: Completor, tag_completor: Completor, account_completor: Completor) -> Self {
        Self{
            focus: PurchaseInputFocus::new(),
            date: DateInput::new(date),
            amount: AmountInput::new(),
            desc: CompletorInput::new('"', '"', false, desc_completor),
            tag: CompletorInput::new('<', '>', true, tag_completor),
            buyer: CompletorInput::new('[', ']', true, account_completor.clone()),
            users: UsersInput::new(account_completor),
        }
    }

    fn input(&mut self, event: InputEvent) {
        use PurchaseInputFocus::*;

        let action = match self.focus {
            Date => self.date.input(event),
            Amount => self.amount.input(event),
            Desc => self.desc.input(event),
            Tag => self.tag.input(event),
            Buyer => self.buyer.input(event),
            Users => self.users.input(event),
        };

        match action {
            Action::Nothing => (),
            Action::Next => self.focus.next(),
            Action::Prev => self.focus.prev(),
        }
    }

    pub fn display(&mut self, line: u16) -> crossterm::Result<()> {
        use PurchaseInputFocus::*;
        let focus = self.focus;

        if focus != Date {self.date.display(line, false)?}
        if focus != Amount {self.amount.display(line+1, false)?}
        if focus != Desc {self.desc.display(line+2, false)?}
        if focus != Tag {self.tag.display(line+3, false)?}
        if focus != Buyer {self.buyer.display(line+4, false)?}
        if focus != Users {self.users.display(line+5, false)?}

        if focus == Date {self.date.display(line, true)?}
        if focus == Amount {self.amount.display(line+1, true)?}
        if focus == Desc {self.desc.display(line+2, true)?}
        if focus == Tag {self.tag.display(line+3, true)?}
        if focus == Buyer {self.buyer.display(line+4, true)?}
        if focus == Users {self.users.display(line+5, true)?}

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

pub fn app() -> crossterm::Result<()> {
    let today: NaiveDate = Local::now().date_naive();

    use crate::tags;
    use crate::accounts;
    use crate::yamlrw::YamlRW;

    let mut tags = tags::Tags::read_yaml("tags.yaml").unwrap();
    tags.fix();

    let accounts = accounts::Accounts::read_yaml("accounts.yaml").unwrap();

    let desc_completor = Completor::new(Vec::new());
    let tag_completor = Completor::new(tags.0.into_keys().collect());
    let account_completor = Completor::new(accounts.0.into_keys().collect());


    let mut transaction = PurchaseInput::new(today, desc_completor, tag_completor, account_completor);

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
