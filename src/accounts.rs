use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

use crate::yamlrw::YamlRW;

pub type AccountRef = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accounts(pub BTreeMap<AccountRef, AccountData>);

impl YamlRW for Accounts {}
