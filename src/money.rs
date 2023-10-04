use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CentsAmount {
    cents: usize,
}

impl CentsAmount {
    pub fn new(cents: usize) -> Self {
        Self{cents}
    }

    pub fn digits(&self) -> Vec<usize> {
        let mut ret = Vec::new();
        let mut value = self.cents;

        while value > 0 {
            ret.push(value%10);
            value /= 10;
        }

        ret
    }

    pub fn as_string_exact(&self, separator: bool) -> String {
        let mut digits = self.digits();
        let scale = 2;
        while digits.len() <= scale {
            digits.push(0);
        }

        let mut ret = Vec::new();
        let mut index = digits.len() - 1;

        loop {
            ret.push(char::from_digit(digits[index].try_into().unwrap(), 10).unwrap());
            if index == 0 {
                break;
            }
            if index == scale {
                ret.push('.');
            } else if index % 3 == 2 && separator {
                ret.push(',');
            }
            index -= 1;
        }

        ret.into_iter().collect()
    }

    pub fn as_string_precision(&self, nb_digits: usize, separator: bool) -> String {
        assert!(nb_digits > 0);
        let mut digits = self.digits();
        if digits.len() <= nb_digits {
            self.as_string_exact(separator)
        } else {
            let smallest_index = digits.len() - nb_digits;
            let (indicator, scale) = if smallest_index <= 2+0 {
                (None, 2+0)
            } else if smallest_index <= 2+3 {
                (Some('k'), 2+3)
            } else if smallest_index <= 2+6 {
                (Some('M'), 2+6)
            } else if smallest_index <= 2+9 {
                (Some('G'), 2+9)
            } else {
                panic!("Amount too big")
            };

            while digits.len() <= scale {
                digits.push(0);
            }

            let mut ret = Vec::new();
            let mut index = digits.len() - 1;

            loop {
                ret.push(char::from_digit(digits[index].try_into().unwrap(), 10).unwrap());
                if index == smallest_index {
                    break;
                }
                if index == scale {
                    ret.push('.');
                } else if index % 3 == 2 && separator {
                    ret.push(',');
                }
                index -= 1;
            }

            if let Some(c) = indicator {
                ret.push(c);
            }

            ret.into_iter().collect()
        }
    }

    pub fn as_string_width(&self, width: usize, separator: bool) -> String {
        let mut nb_digits = 1;
        if self.as_string_exact(separator).len() <= width {
            self.as_string_exact(separator)
        } else {
            while self.as_string_precision(nb_digits+1, separator).len() <= width {
                nb_digits += 1;
            }
            self.as_string_precision(nb_digits, separator)
        }
    }

    pub fn as_string_width_padded(&self, width: usize, separator: bool) -> String {
        format!("{: >width$}", self.as_string_width(width, separator), width = width)
    }
}
