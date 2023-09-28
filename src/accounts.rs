use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::yamlrw::YamlRW;

pub type AccountRef = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accounts(pub HashMap<AccountRef, AccountData>);

impl YamlRW for Accounts {}
