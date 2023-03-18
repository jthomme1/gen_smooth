use std::cmp::{PartialEq, PartialOrd, Ord, Ordering, Eq};
use std::vec::Vec;
use super::PRIMES;

#[derive(Eq)]
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

    fn try_inc_ind(&mut self, bound: u128, ind: usize) -> bool {
        self.value *= u128::try_from(PRIMES[ind]).unwrap();
        self.es[ind] += 1;
        if !(self.value <= bound) {
            self.set_e(ind, 0);
            return false
        }
        return true;
    }

    pub fn inc_vec_with_bound(&mut self, bound: u128) -> bool {
        for i in 0..self.es.len() {
            if self.try_inc_ind(bound, i) {
                return true;
            }
        }
        return false;
    }

    pub fn is_x_smooth(&self, x: usize) -> bool {
        !self.es
            .iter()
            .enumerate()
            .any(|(i, &e)| e != 0 && PRIMES[i] > x)
    }

    pub fn one() -> Self {
        Composite{value: 1, es: vec![0]}
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
