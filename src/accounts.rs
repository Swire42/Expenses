use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

use crate::yamlrw::YamlRW;
use crate::color::RGBColor;

pub type AccountRef = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    color: RGBColor,
}

impl AccountData {
    pub fn color(&self) -> RGBColor {
        self.color
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accounts(pub BTreeMap<AccountRef, AccountData>);

impl YamlRW for Accounts {}
