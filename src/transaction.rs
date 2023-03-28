use chrono::{NaiveDate};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::tags::Tag;
use crate::accounts::AccountRef;
use crate::money::Amount;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumers(HashMap<AccountRef, f64>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Purchase {
    date: NaiveDate,
    amount: Amount,
    desc: String,
    tag: Tag,
    buyer: AccountRef,
    users: Consumers,
}
