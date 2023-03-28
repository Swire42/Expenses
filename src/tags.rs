use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::yamlrw::YamlRW;

pub type TagRef = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagData {
    dur: Option<usize>,
    parent: Option<TagRef>,
}

impl TagData {
    pub fn new() -> Self {
        Self{dur: None, parent: None}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tags(pub HashMap<TagRef, TagData>);

impl YamlRW for Tags {}

impl Tags {
    pub fn fix(&mut self) {
        for data in self.0.clone().into_values() {
            if let TagData{parent: Some(parent), ..} = data {
                if !self.0.contains_key(&parent) {
                    self.0.insert(parent, TagData::new());
                }
            }
        }
    }
}
