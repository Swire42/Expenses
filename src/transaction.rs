use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

use crate::tags::*;
use crate::accounts::AccountRef;
use crate::money::*;
use crate::moneystate::*;
use crate::datetime::Date;
use crate::yamlrw::YamlRW;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Consumers(pub BTreeMap<AccountRef, usize>);

impl Consumers {
    pub fn amounts(&self, total: CentsAmount) -> BTreeMap<AccountRef, CentsAmount> {
        let amounts = total.subdiv(self.0.values().cloned().collect());
        self.0.keys().cloned().zip(amounts).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Purchase {
    pub date: Date,
    pub amount: CentsAmount,
    pub desc: String,
    pub tag: TagRef,
    pub buyer: AccountRef,
    pub consumers: Consumers,
}

impl Purchase {
    pub fn internal_delta(&self, account: &AccountRef) -> SignedCentsAmount {
        self.consumers.amounts(self.amount).get(account).cloned().map(|x| SignedCentsAmount::negative(x)).unwrap_or(SignedCentsAmount::new(0))
    }

    pub fn external_delta(&self, account: &AccountRef) -> SignedCentsAmount {
        if &self.buyer == account {
            SignedCentsAmount::positive(self.amount) + self.internal_delta(account)
        } else {
            self.internal_delta(account)
        }
    }

    pub fn internal_flow(&self, account: &AccountRef, tags: &Tags, transactions: &Transactions) -> SignedFlow {
        SignedFlow::approx(self.internal_delta(account), &transactions.snapshot_before(&self.date, account, tags).state().0[&self.tag], &transactions.snapshot_after(&self.date, account, tags).state().0[&self.tag])
    }
}



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Transaction {
    Purchase(Purchase),
}

impl Transaction {
    pub fn date(&self) -> &Date {
        match &self {
            Transaction::Purchase(purchase) => &purchase.date,
        }
    }

    pub fn abs_amount(&self) -> CentsAmount {
        match &self {
            Transaction::Purchase(purchase) => purchase.amount,
        }
    }

    pub fn internal_delta(&self, account: &AccountRef) -> SignedCentsAmount {
        match &self {
            Transaction::Purchase(purchase) => purchase.internal_delta(account),
        }
    }

    pub fn external_delta(&self, account: &AccountRef) -> SignedCentsAmount {
        match &self {
            Transaction::Purchase(purchase) => purchase.external_delta(account),
        }
    }

    pub fn internal_flow(&self, account: &AccountRef, tags: &Tags, transactions: &Transactions) -> SignedFlow {
        match &self {
            Transaction::Purchase(purchase) => purchase.internal_flow(account, tags, transactions),
        }
    }

    pub fn accounts(&self) -> Vec<AccountRef> {
        let mut ret = Vec::new();
        match &self {
            Transaction::Purchase(purchase) => {
                ret.push(purchase.buyer.clone());
                ret.append(&mut purchase.consumers.0.keys().cloned().filter(|x| x != &purchase.buyer).collect());
            },
        }
        ret
    }

    pub fn desc(&self) -> &String {
        match &self {
            Transaction::Purchase(purchase) => &purchase.desc,
        }
    }

    pub fn kind_str(&self) -> String {
        match &self {
            Transaction::Purchase(purchase) => format!("{}", purchase.tag),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transactions(Vec<Transaction>);

impl Transactions {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn fix(&mut self) {
        self.0.sort_by(|a, b| a.date().cmp(b.date()));
    }

    pub fn add(&mut self, transaction: Transaction) -> usize {
        let index = self.0.partition_point(|tr| tr.date() <= transaction.date());
        self.0.insert(index, transaction);
        index
    }

    pub fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn vec(&self) -> &Vec<Transaction> {
        &self.0
    }

    pub fn initial_snapshot(&self, tags: &Tags) -> FlowStatesSnapshot {
        FlowStatesSnapshot::new(self.0.get(0).map_or_else(|| Date::today(), |x| x.date().clone()), tags)
    }

    pub fn snapshot_before(&self, date: &Date, account: &AccountRef, tags: &Tags) -> FlowStatesSnapshot {
        let mut ret = self.initial_snapshot(tags);

        for tr in &self.0 {
            if tr.date() >= date {
                break;
            }

            match tr {
                Transaction::Purchase(purchase) => {
                    ret.add(purchase, account, tags);
                },
            }
        }

        ret.forward(&date);

        ret
    }

    pub fn snapshot_after(&self, date: &Date, account: &AccountRef, tags: &Tags) -> FlowStatesSnapshot {
        let mut ret = self.initial_snapshot(tags);

        for tr in &self.0 {
            if tr.date() > date {
                break;
            }

            match tr {
                Transaction::Purchase(purchase) => {
                    ret.add(purchase, account, tags);
                },
            }
        }

        ret.forward(date);

        ret
    }
}

impl YamlRW for Transactions {}
