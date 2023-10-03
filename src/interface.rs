use std::io::{stdout, Write};
use std::fmt;
use chrono::{Local, NaiveDate};

use crate::term::*;
use crate::money::Amount;
use crate::completion::Completor;
use crate::transaction::{Transactions, Transaction, Purchase, Consumers};
use crate::tags::Tags;
use crate::accounts::Accounts;

#[derive(Clone)]
pub struct DateInput {
    date: NaiveDate,
}

impl DateInput {
    pub fn new(date: NaiveDate) -> Self {
        Self{date}
    }
}

impl TermElement for DateInput {
    fn display(&self, element_box: TermBox, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            style::{PrintStyledContent, Stylize}
        };

        let mut tmp = self.date.format("%d-%m-%Y").to_string().bold();
        if active {
            tmp = tmp.reverse();
        }
        element_box.begin().goto()?;
        queue!(stdout(), PrintStyledContent(tmp))?;

        Ok(())
    }

    fn popup(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        Ok(())
    }

    fn set_cursor(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        use crossterm::{queue, cursor};
        queue!(stdout(), crossterm::cursor::Hide)
    }

    fn input(&mut self, event: InputEvent) -> Option<InputEvent> {
        use InputEvent::*;
        match event {
            Tab | Enter | BackTab => Some(event),
            Down | Right => {
                self.date = self.date.succ_opt().unwrap();
                None
            },
            Up | Left => {
                self.date = self.date.pred_opt().unwrap();
                None
            },
            _ => None,
        }
    }
}

impl fmt::Display for DateInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.date.format("%d-%m-%Y"))
    }
}

impl From<DateInput> for NaiveDate {
    fn from(date: DateInput) -> NaiveDate {
        date.date
    }
}



#[derive(Clone)]
pub struct AmountInput {
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

    pub fn valid(&self) -> bool {
        self.cents != 0
    }

    pub fn len(&self) -> usize {
        format!("{self}").len()
    }
}

impl TermElement for AmountInput {
    fn display(&self, element_box: TermBox, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            style::{PrintStyledContent, Stylize}
        };

        let mut tmp = format!("{self}").bold();
        if active {
            tmp = tmp.reverse();
        }

        element_box.begin().goto()?;
        queue!(stdout(), PrintStyledContent(tmp))?;

        Ok(())
    }

    fn popup(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        Ok(())
    }

    fn set_cursor(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        use crossterm::{queue, cursor};
        TermPos::new(element_box.left + self.len() - 2, element_box.top).goto()?;
        queue!(stdout(), cursor::Show, cursor::SetCursorStyle::BlinkingBar)
    }

    fn input(&mut self, event: InputEvent) -> Option<InputEvent> {
        use InputEvent::*;

        match event {
            Tab | Enter | BackTab => Some(event),
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
                None
            }
            Char(c) => {
                if let Some(Ok(val)) = c.to_digit(10).map(usize::try_from) {
                    match self.separator_dist {
                        None => {
                            self.cents = self.cents * 10 + val * 100;
                            None
                        },
                        Some(0) => {
                            self.cents += val * 10;
                            self.separator_dist = Some(1);
                            None
                        },
                        Some(1) => {
                            self.cents += val;
                            self.separator_dist = Some(2);
                            Some(Tab)
                        },
                        Some(2) => Some(Tab),
                        _ => unreachable!(),
                    }
                } else if c == '.' || c == ',' {
                    if self.separator_dist == None {
                        self.separator_dist = Some(0);
                        None
                    } else {
                        Some(Tab)
                    }
                } else {
                    None
                }
            },
            _ => None,
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

impl From<AmountInput> for Amount {
    fn from(amount: AmountInput) -> Amount {
        Amount{cents: amount.cents}
    }
}



#[derive(Clone)]
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

    pub fn valid(&self) -> bool {
        !self.is_empty() && (!self.strict || self.compl.contains(&self.text))
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
}

impl TermElement for CompletorInput {
    fn display(&self, element_box: TermBox, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            style::{PrintStyledContent, Stylize}
        };

        let mut tmp: crossterm::style::StyledContent<String> = format!("{}{}{}", self.decor_prefix, self.text, self.decor_suffix).bold();
        if active && self.selection.is_none() {
            tmp = tmp.reverse();
        }
        element_box.begin().goto();
        queue!(stdout(), PrintStyledContent(tmp))?;

        Ok(())
    }

    fn popup(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            style::{PrintStyledContent, Stylize}
        };

        for (lig, (n, sugg)) in ((element_box.top+1)..(window_box.bottom)).zip(self.compl.matches().iter().enumerate()) {
            let tmp = if Some(n) == self.selection {
                format!(">{}<", sugg).bold().reverse()
            } else {
                format!(" {} ", sugg).bold()
            };
            TermPos::new(element_box.left, lig).goto();
            queue!(stdout(), PrintStyledContent(tmp))?;
        }
        Ok(())
    }

    fn set_cursor(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        use crossterm::{queue, cursor};
        TermPos::new(element_box.left + self.text.chars().count() + 1, element_box.top).goto()?;
        if self.selection.is_none() {
            queue!(stdout(), cursor::Show, cursor::SetCursorStyle::BlinkingBar)?;
        } else {
            queue!(stdout(), cursor::Show, cursor::SetCursorStyle::SteadyBar)?;
        }
        Ok(())
    }

    fn input(&mut self, event: InputEvent) -> Option<InputEvent> {
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
            Tab | Enter | BackTab=> {
                self.exit();
                return Some(event);
            },
            _ => (),
        }

        None
    }
}

impl From<CompletorInput> for String {
    fn from(input: CompletorInput) -> String {
        input.text
    }
}



#[derive(Clone)]
pub struct UsersInput {
    new_user: CompletorInput,
    users: Vec<String>,
    selection: Option<usize>,
}

impl UsersInput {
    pub fn new(compl: Completor) -> Self {
        Self{new_user: CompletorInput::new('[', ']', true, compl), users: Vec::new(), selection: None}
    }

    pub fn valid(&self) -> bool {
        !self.users.is_empty()
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

    fn new_user_box(&self, element_box: TermBox) -> TermBox {
        TermBox{left: element_box.left, right: element_box.left+self.new_user.display_len(), top: element_box.top, bottom: element_box.top+1}
    }
}

impl TermElement for UsersInput {
    fn display(&self, element_box: TermBox, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            style::{Print, PrintStyledContent, Stylize}
        };

        self.new_user.display(self.new_user_box(element_box), active && self.selection.is_none())?;

        TermPos::new(element_box.left+self.new_user.display_len(), element_box.top).goto()?;

        for (n, user) in self.users.iter().enumerate() {
            let mut tmp: crossterm::style::StyledContent<String> = format!("{}", user).bold();
            if active && self.selection == Some(n) {
                tmp = tmp.reverse();
            }
            queue!(stdout(), Print(" "), PrintStyledContent(tmp))?;
        }

        Ok(())
    }

    fn popup(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        if self.selection.is_none() {
            self.new_user.popup(self.new_user_box(element_box), window_box)?;
        }
        Ok(())
    }

    fn set_cursor(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        use crossterm::{queue, cursor};
        if self.selection.is_none() {
            self.new_user.set_cursor(self.new_user_box(element_box), window_box)?;
        } else {
            queue!(stdout(), cursor::Hide)?;
        }
        Ok(())
    }

    fn input(&mut self, event: InputEvent) -> Option<InputEvent> {
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
                            Some(event @ (Tab | Enter | BackTab)) => {
                                if self.new_user.is_empty() {
                                    Some(event)
                                } else {
                                    self.validate_new_user();
                                    None
                                }
                            },
                            event => event,
                        };
                    },
                    Some(_) => {
                        match event {
                            Backspace | Delete => {
                                self.del_user();
                            },
                            Tab | Enter | BackTab => {
                                self.exit();
                                return Some(event);
                            },
                            _ => (),
                        }
                    },
                }
            },
        }

        None
    }
}

impl From<UsersInput> for Consumers {
    fn from(users: UsersInput) -> Consumers {
        Consumers(users.users.into_iter().map(|user| (user, 1)).collect())
    }
}



#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PurchaseInputFocus {
    Date,
    Amount,
    Desc,
    Tag,
    Buyer,
    Consumers,
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
            Tag => Buyer,
            Buyer => Consumers,
            Consumers => Date,
        }
    }

    pub fn prev(&mut self) {
        use PurchaseInputFocus::*;
        *self = match self {
            Date => Consumers,
            Amount => Date,
            Desc => Amount,
            Tag => Desc,
            Buyer => Tag,
            Consumers => Buyer,
        }
    }

    pub fn last(&self) -> bool {
        use PurchaseInputFocus::*;
        self == &Consumers
    }
}

#[derive(Clone)]
pub struct PurchaseInput {
    focus: PurchaseInputFocus,
    date: DateInput,
    amount: AmountInput,
    desc: CompletorInput,
    tag: CompletorInput,
    buyer: CompletorInput,
    consumers: UsersInput,
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
            consumers: UsersInput::new(account_completor),
        }
    }

    pub fn valid(&self) -> bool {
        self.amount.valid() && self.desc.valid() && self.tag.valid() && self.buyer.valid() && self.consumers.valid()
    }

    fn child(&self, index: PurchaseInputFocus) -> &dyn TermElement {
        use PurchaseInputFocus::*;

        match index {
            Date      =>      &self.date,
            Amount    =>    &self.amount,
            Desc      =>      &self.desc,
            Tag       =>       &self.tag,
            Buyer     =>     &self.buyer,
            Consumers => &self.consumers,
        }
    }

    fn child_box(&self, index: PurchaseInputFocus, element_box: TermBox) -> TermBox {
        use PurchaseInputFocus::*;

        match index {
            Date      => TermBox{left: element_box.left, right: element_box.right, top: element_box.top+0, bottom: element_box.top+1},
            Amount    => TermBox{left: element_box.left, right: element_box.right, top: element_box.top+1, bottom: element_box.top+2},
            Desc      => TermBox{left: element_box.left, right: element_box.right, top: element_box.top+2, bottom: element_box.top+3},
            Tag       => TermBox{left: element_box.left, right: element_box.right, top: element_box.top+3, bottom: element_box.top+4},
            Buyer     => TermBox{left: element_box.left, right: element_box.right, top: element_box.top+4, bottom: element_box.top+5},
            Consumers => TermBox{left: element_box.left, right: element_box.right, top: element_box.top+5, bottom: element_box.top+6},
        }
    }
}

impl TermElement for PurchaseInput {
    fn display(&self, element_box: TermBox, active: bool) -> crossterm::Result<()> {
        use PurchaseInputFocus::*;

        for index in [Date, Amount, Desc, Tag, Buyer, Consumers] {
            self.child(index).display(self.child_box(index, element_box), index == self.focus)?;
        }

        Ok(())
    }

    fn popup(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        self.child(self.focus).popup(self.child_box(self.focus, element_box), window_box)
    }

    fn set_cursor(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        self.child(self.focus).set_cursor(self.child_box(self.focus, element_box), window_box)
    }

    fn input(&mut self, event: InputEvent) -> Option<InputEvent> {
        use PurchaseInputFocus::*;

        let event_opt = match self.focus {
            Date      =>      self.date.input(event),
            Amount    =>    self.amount.input(event),
            Desc      =>      self.desc.input(event),
            Tag       =>       self.tag.input(event),
            Buyer     =>     self.buyer.input(event),
            Consumers => self.consumers.input(event),
        };

        use InputEvent::*;

        if let Some(event) = event_opt {
            match event {
                Tab | Enter => {
                    if self.focus.last() && self.valid() {
                        return Some(event);
                    }
                    self.focus.next();
                },
                BackTab => self.focus.prev(),
                _ => (),
            }
        }

        None
    }
}

impl From<PurchaseInput> for Purchase {
    fn from(purchase: PurchaseInput) -> Purchase {
        Purchase {
            date: purchase.date.into(),
            amount: purchase.amount.into(),
            desc: purchase.desc.into(),
            tag: purchase.tag.into(),
            buyer: purchase.buyer.into(),
            consumers: purchase.consumers.into(),
        }
    }
}



#[derive(Clone)]
pub struct App {
    tags: Tags,
    accounts: Accounts,
    transactions: Transactions,
    purchase: Option<PurchaseInput>,
}

impl App {
    pub fn new() -> Self {
        use crate::yamlrw::YamlRW;

        let today: NaiveDate = Local::now().date_naive();

        let mut tags = Tags::read_yaml("tags.yaml").unwrap();
        tags.fix();

        let accounts = Accounts::read_yaml("accounts.yaml").unwrap();

        let mut transactions = Transactions::read_yaml("data.yaml").unwrap_or_else(|_| Transactions::new());

        let mut ret = Self{tags, accounts, transactions, purchase: None};
        ret.new_purchase(today);
        ret
    }

    fn new_purchase(&mut self, date: NaiveDate) {
        let desc_completor = Completor::new(Vec::new());
        let tag_completor = Completor::new(self.tags.clone().0.into_keys().collect());
        let account_completor = Completor::new(self.accounts.clone().0.into_keys().collect());

        self.purchase = Some(PurchaseInput::new(date, desc_completor, tag_completor, account_completor));
    }

    fn child_box(&self, element_box: TermBox) -> TermBox {
        TermBox{left: element_box.left, right: element_box.right, top: element_box.top+2, bottom: element_box.bottom}
    }
}

impl Drop for App {
    fn drop(&mut self) {
        use crate::yamlrw::YamlRW;
        self.transactions.write_yaml("data.yaml").unwrap();
    }
}

impl TermElement for App {
    fn display(&self, element_box: TermBox, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            style::{Print},
        };

        element_box.begin().goto()?;
        queue!(stdout(), Print("Hello"))?;

        match &self.purchase {
            Some(purchase) => {
                purchase.display(self.child_box(element_box), true)?;
            },
            _ => (),
        }

        Ok(())
    }

    fn popup(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        match &self.purchase {
            Some(purchase) => purchase.popup(self.child_box(element_box), window_box),
            _ => Ok(()),
        }
    }

    fn set_cursor(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        match &self.purchase {
            Some(purchase) => purchase.set_cursor(self.child_box(element_box), window_box),
            _ => Ok(()),
        }
    }

    fn input(&mut self, event: InputEvent) -> Option<InputEvent> {
        use InputEvent::*;

        match &mut self.purchase {
            Some(purchase) => {
                match event {
                    Esc => Some(Esc),
                    _ => {
                        match purchase.input(event) {
                            Some(Tab | Enter) => {
                                let date = purchase.date.date.clone();
                                self.transactions.add(Transaction::Purchase(purchase.clone().into()));
                                self.new_purchase(date);
                            },
                            _ => (),
                        }
                        None
                    },
                }
            },
            _ => {
                match event {
                    Esc => Some(Esc),
                    _ => None,
                }
            },
        }
    }
}
