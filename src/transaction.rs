use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

use crate::tags::TagRef;
use crate::accounts::AccountRef;
use crate::money::*;
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

    pub fn internal_balance(&self, account: &AccountRef) -> SignedCentsAmount {
        match &self {
            Transaction::Purchase(purchase) => purchase.consumers.amounts(purchase.amount).get(account).cloned().map(|x| SignedCentsAmount::negative(x)).unwrap_or(SignedCentsAmount::new(0)),
        }
    }

    pub fn external_balance(&self, account: &AccountRef) -> SignedCentsAmount {
        match &self {
            Transaction::Purchase(purchase) => {
                if &purchase.buyer == account {
                    SignedCentsAmount::positive(purchase.amount) + self.internal_balance(account)
                } else {
                    self.internal_balance(account)
                }
            },
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
}

impl YamlRW for Transactions {}
