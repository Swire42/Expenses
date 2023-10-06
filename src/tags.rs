use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

use crate::yamlrw::YamlRW;

pub type TagRef = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagData {
    dur: usize,
    parent: Option<TagRef>,
}

impl TagData {
    pub fn new(dur: usize) -> Self {
        Self{dur, parent: None}
    }

    pub fn dur(&self) -> usize {
        self.dur
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tags(pub BTreeMap<TagRef, TagData>);

impl YamlRW for Tags {}

impl Tags {
    pub fn fix(&mut self) {
        for data in self.0.clone().into_values() {
            if let TagData{dur, parent: Some(parent)} = data {
                if !self.0.contains_key(&parent) {
                    self.0.insert(parent, TagData::new(dur));
                }
            }
        }
    }
}
