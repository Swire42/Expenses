use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::yamlrw::YamlRW;

pub type AccountRef = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    name: AccountRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accounts(HashMap<AccountRef, Account>);

impl YamlRW for Accounts {}
