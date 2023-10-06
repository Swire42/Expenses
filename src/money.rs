use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CentsAmount {
    cents: u64,
}

impl CentsAmount {
    pub fn new(cents: u64) -> Self {
        Self{cents}
    }

    pub fn cents(&self) -> u64 {
        self.cents
    }

    pub fn digits(&self) -> Vec<usize> {
        let mut ret = Vec::new();
        let mut value = self.cents;

        while value > 0 {
            ret.push((value%10) as usize);
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

    pub fn subdiv(&self, weights: Vec<usize>) -> Vec<Self> {
        assert!(!weights.is_empty());
        let wsum: u64 = weights.iter().sum::<usize>() as u64;
        let mut ret: Vec<u64> = weights.iter().map(|w| self.cents * *w as u64 / wsum).collect();
        let ret_sum: u64 = ret.iter().sum();
        let rem = (self.cents - ret_sum) as usize;
        for k in 0..rem {
            ret[k] += 1;
        }
        ret.into_iter().map(|c| Self::new(c)).collect()
    }
}

impl std::ops::Add for CentsAmount {
    type Output = CentsAmount;

    fn add(self, other: Self) -> Self {
        Self{cents: self.cents + other.cents}
    }
}

impl std::ops::AddAssign for CentsAmount {
    fn add_assign(&mut self, other: Self) {
        self.cents += other.cents;
    }
}

impl std::ops::Sub for CentsAmount {
    type Output = CentsAmount;

    fn sub(self, other: Self) -> Self {
        Self{cents: self.cents - other.cents}
    }
}

impl std::ops::SubAssign for CentsAmount {
    fn sub_assign(&mut self, other: Self) {
        self.cents -= other.cents;
    }
}

impl std::ops::Div<usize> for CentsAmount {
    type Output = CentsAmount;

    fn div(self, other: usize) -> Self {
        Self{cents: self.cents / other as u64}
    }
}

impl std::ops::Div<CentsAmount> for CentsAmount {
    type Output = usize;

    fn div(self, other: CentsAmount) -> usize {
        (self.cents / other.cents) as usize
    }
}



#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedCentsAmount {
    cents: i64,
}

impl SignedCentsAmount {
    pub fn new(cents: i64) -> Self {
        Self{cents}
    }

    pub fn cents(&self) -> i64 {
        self.cents
    }

    pub fn positive(amount: CentsAmount) -> Self {
        Self{cents: amount.cents as i64}
    }

    pub fn negative(amount: CentsAmount) -> Self {
        Self{cents: -(amount.cents as i64)}
    }

    pub fn abs(&self) -> CentsAmount {
        CentsAmount{cents: self.cents.abs_diff(0)}
    }

    pub fn as_string_exact(&self, separator: bool) -> String {
        use std::cmp::Ordering::*;

        match self.cents.cmp(&0) {
            Less => format!("-{}", self.abs().as_string_exact(separator)),
            Equal => format!("{}", self.abs().as_string_exact(separator)),
            Greater => format!("+{}", self.abs().as_string_exact(separator)),
        }
    }

    pub fn as_string_precision(&self, nb_digits: usize, separator: bool) -> String {
        use std::cmp::Ordering::*;

        match self.cents.cmp(&0) {
            Less => format!("-{}", self.abs().as_string_precision(nb_digits, separator)),
            Equal => format!("{}", self.abs().as_string_precision(nb_digits, separator)),
            Greater => format!("+{}", self.abs().as_string_precision(nb_digits, separator)),
        }
    }

    pub fn as_string_width(&self, width: usize, separator: bool) -> String {
        use std::cmp::Ordering::*;

        match self.cents.cmp(&0) {
            Less => format!("-{}", self.abs().as_string_width(width-1, separator)),
            Equal => format!("{}", self.abs().as_string_width(width-1, separator)),
            Greater => format!("+{}", self.abs().as_string_width(width-1, separator)),
        }
    }

    pub fn as_string_width_padded(&self, width: usize, separator: bool) -> String {
        format!("{: >width$}", self.as_string_width(width, separator), width = width)
    }
}

impl std::ops::Add for SignedCentsAmount {
    type Output = SignedCentsAmount;

    fn add(self, other: Self) -> Self {
        Self{cents: self.cents + other.cents}
    }
}
