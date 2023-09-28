use chrono::{NaiveDate};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::tags::TagRef;
use crate::accounts::AccountRef;
use crate::money::Amount;
use crate::yamlrw::YamlRW;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumers(pub HashMap<AccountRef, usize>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purchase {
    pub date: NaiveDate,
    pub amount: Amount,
    pub desc: String,
    pub tag: TagRef,
    pub buyer: AccountRef,
    pub consumers: Consumers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transaction {
    Purchase(Purchase),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transactions(Vec<Transaction>);

impl Transactions {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, transaction: Transaction) {
        self.0.push(transaction);
    }
}

impl YamlRW for Transactions {}
