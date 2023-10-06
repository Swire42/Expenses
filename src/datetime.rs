use chrono::{Local, NaiveDate};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Date {
    date: NaiveDate,
}

impl Date {
    pub const STRING_WIDTH: usize = 2+1+2+1+4;

    pub fn today() -> Self {
        Self{date: Local::now().date_naive()}
    }

    pub fn to_string(&self) -> String {
        self.date.format("%d-%m-%Y").to_string()
    }

    pub fn succ(&self) -> Self {
        Self{date: self.date.succ_opt().unwrap()}
    }

    pub fn pred(&self) -> Self {
        Self{date: self.date.pred_opt().unwrap()}
    }

    pub fn incr(&mut self) {
        *self = self.succ();
    }

    pub fn decr(&mut self) {
        *self = self.pred();
    }
}
