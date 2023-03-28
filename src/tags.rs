use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::yamlrw::YamlRW;

pub type Tag = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tags(HashMap<Tag, Vec<Tag>>);

impl YamlRW for Tags {}
