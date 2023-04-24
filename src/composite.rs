use std::cmp::{PartialEq, PartialOrd, Ord, Ordering, Eq};
use std::vec::Vec;
use super::PRIMES;

#[derive(Eq, Debug)]
pub struct Composite {
    pub value: u128,
    pub es: Vec<u32>,
}

impl Composite {
    pub fn new(es: Vec<u32>) -> Self {
        assert!(es.len() <= PRIMES.len());
        let value = es
            .iter()
            .enumerate()
            .fold(1u128, |acc, (i, &e)| acc*u128::try_from(PRIMES[i]).unwrap().pow(e));
        Composite{value, es}
    }

    fn set_e(&mut self, ind: usize, new_e: u32) {
        let old_e = self.es[ind];
        if old_e > new_e {
            let change = u128::try_from(PRIMES[ind]).unwrap().pow(old_e - new_e);
            self.value /= change
        } else {
            let change = u128::try_from(PRIMES[ind]).unwrap().pow(new_e - old_e);
            self.value *= change
        }
        self.es[ind] = new_e;
    }

    fn try_inc_ind_rst(&mut self, bound: u128, ind: usize, e: u32) -> Option<Self> {
        // try to increment the exponent at index ind and set it to e otherwise
        // returns None on success and Some(composite number) if it surpasses our bound
        self.value *= u128::try_from(PRIMES[ind]).unwrap();
        self.es[ind] += 1;
        if self.value > bound {
            let r = self.clone();
            // this also resets the value correctly
            self.set_e(ind, e);
            return Some(r);
        }
        None
    }

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

    pub fn one(len: usize) -> Self {
        Composite{value: 1, es: vec![0; len]}
    }
}

impl Clone for Composite {
    fn clone(&self) -> Self {
        Composite{value: self.value, es: self.es.clone()}
    }

    fn clone_from(&mut self, source: &Self) {
        self.value = source.value;
        self.es = source.es.clone();
    }
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
