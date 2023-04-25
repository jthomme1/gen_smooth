use std::cmp::{PartialEq, PartialOrd, Ord, Ordering, Eq};
use std::fmt;
use super::PRIMES;

const NUMBER_EXP: usize = 64;

#[derive(Eq, Copy, Clone)]
pub struct Composite {
    pub value: u128,
    pub es: [u8; NUMBER_EXP],
}

impl Composite {
    pub fn new(e_ind: usize, e_val: u8) -> Self {
        let mut es: [u8; NUMBER_EXP] = [0; NUMBER_EXP];
        es[e_ind] = e_val;
        let value = es
            .iter()
            .enumerate()
            .fold(1u128, |acc, (i, &e)| acc*u128::try_from(PRIMES[i]).unwrap().pow(e.into()));
        Composite{value, es}
    }

    fn set_e(&mut self, ind: usize, new_e: u8) {
        let old_e = self.es[ind];
        if old_e > new_e {
            let change = u128::try_from(PRIMES[ind]).unwrap().pow((old_e - new_e).into());
            self.value /= change
        } else {
            let change = u128::try_from(PRIMES[ind]).unwrap().pow((new_e - old_e).into());
            self.value *= change
        }
        self.es[ind] = new_e;
    }

    pub fn try_inc_ind(&mut self, bound: u128, ind: usize) -> Option<Self> {
        // try to increment the exponent at index ind and set it to 0 otherwise
        // returns None on success and Some(composite number) if it surpasses our bound
        self.value *= u128::try_from(PRIMES[ind]).unwrap();
        self.es[ind] += 1;
        if self.value > bound {
            let r = self.clone();
            // this also resets the value correctly
            self.set_e(ind, 0);
            return Some(r);
        }
        None
    }

    pub fn highest_exp(&self) -> usize {
        let mut i = NUMBER_EXP-1;
        while self.es[i] == 0 {
            i -= 1;
        }
        i
    }
/*
    pub fn inc_vec_with_bound_rst(&mut self, bound: u128, rst: &Self) -> Vec<Self> {
        // increment the number represented by the exponents
        // return the values generated when trying to increment that surpassed the bound
        let mut ret = vec![];
        for i in 0..self.es.len() {
            match self.try_inc_ind_rst(bound, i, rst.es[i]) {
                None => return ret,
                Some(v) => ret.push(v),
            }
        }
        return ret;
    }
    pub fn one() -> Self {
        Self::new(0, 0)
    }
*/
}

impl Ord for Composite {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for Composite {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Composite {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl fmt::Display for Composite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, [{}, {}, {}]", self.value, self.es[0], self.es[1], self.es[2])
    }
}

impl fmt::Debug for Composite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, [{}, {}, {}]", self.value, self.es[0], self.es[1], self.es[2])
    }
}
