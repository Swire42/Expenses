use std::io::{stdout};
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

use crate::term::*;
use crate::money::*;
use crate::datetime::Date;
use crate::completion::Completor;
use crate::transaction::{Transactions, Transaction, Purchase, Consumers};
use crate::tags::Tags;
use crate::accounts::Accounts;

#[derive(Clone)]
pub struct DateInput {
    date: Date,
}

impl DateInput {
    pub fn new(date: Date) -> Self {
        Self{date}
    }
}

impl TermElement for DateInput {
    fn display(&self, element_box: TermBox, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            style::{PrintStyledContent, Stylize}
        };

        let mut tmp = self.date.to_string().bold();
        if active {
            tmp = tmp.reverse();
        }
        element_box.begin().goto()?;
        queue!(stdout(), PrintStyledContent(tmp))?;

        Ok(())
    }

    fn popup(&self, _element_box: TermBox, _window_box: TermBox) -> crossterm::Result<()> {
        Ok(())
    }

    fn set_cursor(&self, _element_box: TermBox, _window_box: TermBox) -> crossterm::Result<()> {
        use crossterm::{queue, cursor};
        queue!(stdout(), cursor::Hide)
    }

    fn input(&mut self, event: InputEvent) -> Option<InputEvent> {
        use InputEvent::*;
        match event {
            Down | Right => {
                self.date = self.date.succ();
                None
            },
            Up | Left => {
                self.date = self.date.pred();
                None
            },
            _ => Some(event),
        }
    }
}

impl fmt::Display for DateInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.date.to_string())
    }
}

impl From<DateInput> for Date {
    fn from(date: DateInput) -> Date {
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

    fn popup(&self, _element_box: TermBox, _window_box: TermBox) -> crossterm::Result<()> {
        Ok(())
    }

    fn set_cursor(&self, element_box: TermBox, _window_box: TermBox) -> crossterm::Result<()> {
        use crossterm::{queue, cursor};
        TermPos::new(element_box.left + self.len() - 2, element_box.top).goto()?;
        queue!(stdout(), cursor::Show, cursor::SetCursorStyle::BlinkingBar)
    }

    fn input(&mut self, event: InputEvent) -> Option<InputEvent> {
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
            _ => Some(event),
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

impl From<AmountInput> for CentsAmount {
    fn from(amount: AmountInput) -> CentsAmount {
        CentsAmount::new(amount.cents)
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
        element_box.begin().goto()?;
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
            TermPos::new(element_box.left, lig).goto()?;
            queue!(stdout(), PrintStyledContent(tmp))?;
        }
        Ok(())
    }

    fn set_cursor(&self, element_box: TermBox, _window_box: TermBox) -> crossterm::Result<()> {
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
                None
            },
            Char(c) => {
                self.text.push(c);
                self.compl.update(&self.text);
                self.selection = None;
                if self.strict && self.compl.matches().is_empty() {
                    let _ = self.text.pop();
                    self.compl.update(&self.text);
                }
                None
            },
            Down => {
                if !self.compl.matches().is_empty() {
                    self.selection = Some(self.selection.map_or(0, |x| usize::min(x+1, self.compl.matches().len()-1)));
                }
                None
            },
            Up => {
                self.selection = match self.selection {
                    None | Some(0) => None,
                    Some(x) => Some(x-1),
                };
                None
            },
            Tab | Enter | BackTab=> {
                self.exit();
                Some(event)
            },
            _ => Some(event),
        }
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
                None
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
                None
            },
            event => {
                match &self.selection {
                    None => {
                        match self.new_user.input(event) {
                            Some(event @ (Tab | Enter | BackTab)) => {
                                if self.new_user.is_empty() {
                                    Some(event)
                                } else {
                                    self.validate_new_user();
                                    None
                                }
                            },
                            event => event,
                        }
                    },
                    Some(_) => {
                        match event {
                            Backspace | Delete => {
                                self.del_user();
                                None
                            },
                            Tab | Enter | BackTab => {
                                self.exit();
                                Some(event)
                            },
                            _ => Some(event),
                        }
                    },
                }
            },
        }
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

    pub fn all() -> [Self; 6] {
        use PurchaseInputFocus::*;
        [Date, Amount, Desc, Tag, Buyer, Consumers]
    }

    pub fn count() -> usize {
        Self::all().len()
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
    pub fn new(date: Date, desc_completor: Completor, tag_completor: Completor, account_completor: Completor) -> Self {
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
    fn display(&self, element_box: TermBox, _active: bool) -> crossterm::Result<()> {
        for index in PurchaseInputFocus::all() {
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

        match event_opt {
            Some(Tab | Enter) => {
                if self.focus.last() && self.valid() {
                    event_opt
                } else {
                    self.focus.next();
                    None
                }
            },
            Some(BackTab) => {
                self.focus.prev();
                None
            },
            _ => event_opt,
        }
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
pub struct InteractiveTransactions {
    transactions: Transactions,
    selection: usize,
}

impl InteractiveTransactions {
    pub fn new(transactions: Transactions) -> Self {
        Self{transactions, selection: 0}
    }

    pub fn transactions(&self) -> &Transactions {
        &self.transactions
    }

    pub fn add(&mut self, transaction: Transaction) -> usize {
        let index = self.transactions.add(transaction);
        self.selection = index;
        index
    }
}



#[derive(Clone)]
pub struct TransactionsTE {
    transactions: Rc<RefCell<InteractiveTransactions>>,
}

impl TransactionsTE {
    pub fn new(transactions: Rc<RefCell<InteractiveTransactions>>) -> Self {
        Self{transactions}
    }

    fn display_transaction(transaction: &Transaction, element_box: TermBox, active: bool) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            style::{PrintStyledContent, Color, StyledContent},
        };

        assert_eq!(element_box.height(), 1);

        let date_cf = (Date::STRING_WIDTH, 0);
        let amount_cf = (6, 0);
        let space_cf = (1, 0);
        let desc_cf = (10, 2);
        let kind_cf = (6, 1);

        let [_, _, kind_width, _, desc_width, _, amount_width, _, _] = subdiv_const_flex(element_box.width(), [date_cf, space_cf, kind_cf, space_cf, desc_cf, space_cf, amount_cf, space_cf, amount_cf]);

        element_box.begin().goto()?;

        let space = simple_stylize(" ", Color::Reset, true, active);

        let date = simple_stylize(transaction.date().to_string(), Color::Reset, true, active);

        fn stylize_amount(amount: SignedCentsAmount, width: usize, active: bool) -> StyledContent<String> {
            use std::cmp::Ordering::*;
            let color = match amount.cents().cmp(&0) {
                Less => Color::Red,
                Equal => Color::Reset,
                Greater => Color::Green,
            };
            let bold = amount.cents() != 0;
            simple_stylize(amount.as_string_width_padded(width, false), color, bold, active)
        }

        let int_amount = stylize_amount(transaction.internal_balance(&"Swire".to_string()), amount_width, active);
        let ext_amount = stylize_amount(transaction.external_balance(&"Swire".to_string()), amount_width, active);

        let kind = simple_stylize(truncate_align_left(&transaction.kind_str(), kind_width), Color::Reset, true, active);
        let desc = simple_stylize(truncate_align_left(transaction.desc(), desc_width), Color::Reset, true, active);

        queue!(stdout(), PrintStyledContent(date), PrintStyledContent(space), PrintStyledContent(kind), PrintStyledContent(space), PrintStyledContent(desc), PrintStyledContent(space), PrintStyledContent(int_amount), PrintStyledContent(space), PrintStyledContent(ext_amount))?;

        Ok(())
    }

    fn display_transaction_header(element_box: TermBox) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            style::{PrintStyledContent, Color},
        };

        assert_eq!(element_box.height(), 1);

        let date_cf = (Date::STRING_WIDTH, 0);
        let amount_cf = (6, 0);
        let space_cf = (1, 0);
        let desc_cf = (10, 2);
        let kind_cf = (6, 1);

        let [date_width, _, kind_width, _, desc_width, _, amount_width, _, _] = subdiv_const_flex(element_box.width(), [date_cf, space_cf, kind_cf, space_cf, desc_cf, space_cf, amount_cf, space_cf, amount_cf]);

        element_box.begin().goto()?;

        let space = simple_stylize(" ", Color::Reset, true, false);

        let date = simple_stylize(truncate_align_left("Date", date_width), Color::Reset, true, false);
        let int_amount = simple_stylize(truncate_align_left("Int", amount_width), Color::Reset, true, false);
        let ext_amount = simple_stylize(truncate_align_left("Ext", amount_width), Color::Reset, true, false);
        let kind = simple_stylize(truncate_align_left("Kind", kind_width), Color::Reset, true, false);
        let desc = simple_stylize(truncate_align_left("Desc", desc_width), Color::Reset, true, false);

        queue!(stdout(), PrintStyledContent(date), PrintStyledContent(space), PrintStyledContent(kind), PrintStyledContent(space), PrintStyledContent(desc), PrintStyledContent(space), PrintStyledContent(int_amount), PrintStyledContent(space), PrintStyledContent(ext_amount))?;

        Ok(())
    }
}

impl TermElement for TransactionsTE {
    fn display(&self, element_box: TermBox, _active: bool) -> crossterm::Result<()> {
        use crossterm::{
            queue,
            style::{Print},
        };

        let height = element_box.height();
        assert!(height > 5);
        let list_height = height - 1;

        let center_index = self.transactions.borrow().selection;
        let mut begin_index = center_index;
        let mut end_index = center_index;

        while end_index - begin_index < list_height {
            let avail_begin = begin_index > 0;
            let avail_end = end_index < self.transactions.borrow().transactions().len();

            match (avail_begin, avail_end) {
                (true, true) if center_index - begin_index < end_index - center_index => begin_index -= 1,
                (true, true) => end_index += 1,
                (true, false) => begin_index -= 1,
                (false, true) => end_index += 1,
                (false, false) => break,
            }
        }

        let header_box = TermBox{left: element_box.left, right: element_box.right, top: element_box.top, bottom: element_box.top+1};
        Self::display_transaction_header(header_box)?;

        for (index, transaction) in self.transactions.borrow().transactions().vec()[begin_index..end_index].iter().enumerate() {
            let trans_selected = begin_index + index == self.transactions.borrow().selection;
            let trans_box = TermBox{left: element_box.left, right: element_box.right, top: element_box.top+index+1, bottom: element_box.top+index+2};
            Self::display_transaction(&transaction, trans_box, trans_selected)?;
        }

        Ok(())
    }

    fn popup(&self, _element_box: TermBox, _window_box: TermBox) -> crossterm::Result<()> {
        Ok(())
    }

    fn set_cursor(&self, _element_box: TermBox, _window_box: TermBox) -> crossterm::Result<()> {
        use crossterm::{queue, cursor};
        queue!(stdout(), cursor::Hide)
    }

    fn input(&mut self, event: InputEvent) -> Option<InputEvent> {
        Some(event)
    }
}



#[derive(Clone)]
pub struct AppContent {
    tags: Tags,
    accounts: Accounts,
    transactions: Rc<RefCell<InteractiveTransactions>>,
    transactions_menu: TransactionsTE,
    purchase: Option<PurchaseInput>,
}

impl AppContent {
    pub fn new() -> Self {
        use crate::yamlrw::YamlRW;

        let mut tags = Tags::read_yaml("tags.yaml").unwrap();
        tags.fix();

        let accounts = Accounts::read_yaml("accounts.yaml").unwrap();

        let mut transactions = Transactions::read_yaml("data.yaml").unwrap_or_else(|_| Transactions::new());
        transactions.fix();
        let transactions = Rc::new(RefCell::new(InteractiveTransactions::new(transactions)));

        Self{tags, accounts, transactions: Rc::clone(&transactions), transactions_menu: TransactionsTE::new(transactions), purchase: None}
    }

    fn new_purchase(&mut self, date: Date) {
        let desc_completor = Completor::new(Vec::new());
        let tag_completor = Completor::new(self.tags.clone().0.into_keys().collect());
        let account_completor = Completor::new(self.accounts.clone().0.into_keys().collect());

        self.purchase = Some(PurchaseInput::new(date, desc_completor, tag_completor, account_completor));
    }

    fn child_box(&self, element_box: TermBox) -> TermBox {
        TermBox{left: element_box.left, right: element_box.right, top: element_box.top+2, bottom: element_box.bottom}
    }
}

impl Drop for AppContent {
    fn drop(&mut self) {
        use crate::yamlrw::YamlRW;
        self.transactions.borrow_mut().transactions().write_yaml("data.yaml").unwrap();
    }
}

impl TermElement for AppContent {
    fn display(&self, element_box: TermBox, _active: bool) -> crossterm::Result<()> {
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
            None => {
                self.transactions_menu.display(self.child_box(element_box), true)?;
            },
        }

        Ok(())
    }

    fn popup(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        match &self.purchase {
            Some(purchase) => purchase.popup(self.child_box(element_box), window_box),
            None => self.transactions_menu.popup(self.child_box(element_box), window_box),
        }
    }

    fn set_cursor(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        match &self.purchase {
            Some(purchase) => purchase.set_cursor(self.child_box(element_box), window_box),
            _ => self.transactions_menu.set_cursor(self.child_box(element_box), window_box),
        }
    }

    fn input(&mut self, event: InputEvent) -> Option<InputEvent> {
        use InputEvent::*;

        match &mut self.purchase {
            Some(purchase) => {
                match purchase.input(event) {
                    Some(Tab | Enter) => {
                        let date = purchase.date.date.clone();
                        self.transactions.borrow_mut().add(Transaction::Purchase(purchase.clone().into()));
                        self.new_purchase(date);
                        None
                    },
                    Some(Esc) => {
                        self.purchase = None;
                        None
                    },
                    event_opt => event_opt,
                }
            },
            None => {
                match self.transactions_menu.input(event) {
                    Some(Char('i')) => {
                        self.new_purchase(Date::today());
                        None
                    },
                    event_opt => event_opt,
                }
            },
        }
    }
}



pub struct App(AppContent);

impl App {
    pub fn new() -> Self {
        Self(AppContent::new())
    }
}

impl TermElement for App {
    fn display(&self, element_box: TermBox, active: bool) -> crossterm::Result<()> {
        self.0.display(element_box, active)
    }

    fn popup(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        self.0.popup(element_box, window_box)
    }

    fn set_cursor(&self, element_box: TermBox, window_box: TermBox) -> crossterm::Result<()> {
        self.0.set_cursor(element_box, window_box)
    }

    fn input(&mut self, event: InputEvent) -> Option<InputEvent> {
        use InputEvent::*;

        match self.0.input(event) {
            Some(Esc) => Some(Esc),
            _ => None,
        }
    }
}
