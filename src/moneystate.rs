use std::collections::BTreeMap;

use crate::money::*;
use crate::accounts::AccountRef;
use crate::tags::*;
use crate::transaction::*;
use crate::datetime::*;

#[derive(Debug, Copy, Clone)]
pub struct Balance {
    external: SignedCentsAmount,
}

#[derive(Debug, Clone)]
pub struct Balances(pub BTreeMap<AccountRef, Balance>);



#[derive(Debug, Copy, Clone)]
pub struct Flow(pub CentsAmount);

impl Flow {
    pub fn approx(amount: CentsAmount, without: &FlowState, with: &FlowState) -> Self {
        if amount.cents() == 0 {
            Self(CentsAmount::new(0))
        } else {
            Self(CentsAmount::new(amount.cents() * with.flow().0.cents() / (with.amount.cents() - without.amount.cents())))
        }
    }
}



#[derive(Debug, Copy, Clone)]
pub struct SignedFlow(pub SignedCentsAmount);

impl SignedFlow {
    pub fn approx(amount: SignedCentsAmount, without: &FlowState, with: &FlowState) -> Self {
        if amount.cents() == 0 {
            Self(SignedCentsAmount::new(0))
        } else {
            Self(SignedCentsAmount::new(amount.cents() * with.flow().0.cents() as i64 / (with.amount.cents() - without.amount.cents()) as i64))
        }
    }
}



#[derive(Debug, Copy, Clone)]
pub struct FlowState {
    amount: CentsAmount,
    days: usize,
}

impl FlowState {
    pub fn new() -> Self {
        Self{amount: CentsAmount::new(0), days: 0}
    }

    pub fn inactive(&self) -> bool {
        assert_eq!(self.amount.cents() == 0, self.days == 0);
        self.days == 0
    }

    pub fn flow(&self) -> Flow {
        if self.inactive() {
            return Flow(CentsAmount::new(0));
        }

        let absorbed = self.amount / self.days;

        Flow(absorbed)
    }

    pub fn step(&mut self) -> Flow {
        if self.inactive() {
            return Flow(CentsAmount::new(0));
        }

        let absorbed = self.amount / self.days;
        self.amount -= absorbed;
        self.days -= 1;

        Flow(absorbed)
    }

    pub fn next(&self) -> Self {
        let mut ret = self.clone();
        ret.step();
        ret
    }

    pub fn add(&mut self, amount: CentsAmount, dur: usize) {
        if amount.cents() == 0 {
            return;
        }
        assert!(dur > 0);
        self.amount += amount;
        self.days = dur;
    }
}



pub struct FlowStates(pub BTreeMap<TagRef, FlowState>);

impl FlowStates {
    pub fn new(tags: &Tags) -> Self {
        Self(tags.0.keys().cloned().map(|tag| (tag, FlowState::new())).collect())
    }

    pub fn step(&mut self) {
        self.0.values_mut().for_each(|x| { x.step(); });
    }

    pub fn add(&mut self, purchase: &Purchase, account: &AccountRef, tags: &Tags) {
        let tag = &purchase.tag;
        let dur = tags.0.get(tag).unwrap().dur();
        let amount = purchase.internal_delta(account).abs();

        self.0.get_mut(tag).unwrap().add(amount, dur);
    }
}



pub struct FlowStatesSnapshot {
    date: Date,
    state: FlowStates,
}

impl FlowStatesSnapshot {
    pub fn new(date: Date, tags: &Tags) -> Self {
        let state = FlowStates::new(tags);
        Self{date, state}
    }

    pub fn date(&self) -> &Date {
        &self.date
    }

    pub fn state(&self) -> &FlowStates {
        &self.state
    }

    pub fn step(&mut self) {
        self.state.step();
        self.date.incr();
    }

    pub fn add(&mut self, purchase: &Purchase, account: &AccountRef, tags: &Tags) {
        let date = &purchase.date;
        assert!(date >= &self.date);
        while date > &self.date {
            self.step();
        }
        self.state.add(purchase, account, tags);
    }

    pub fn forward(&mut self, date: &Date) {
        assert!(date >= &self.date);
        while date > &self.date {
            self.step();
        }
    }
}
