use std::cmp::{PartialEq, PartialOrd, Ord, Ordering, Eq};
use std::vec::Vec;
use super::PRIMES;

#[derive(Eq, Debug)]
pub struct Composite {
    pub value: u64,
    pub es: Vec<u32>,
}

impl Composite {
    pub fn new(e_ind: usize, e_val: u32) -> Self {
        let mut es: Vec<u32> = vec![0; e_ind+1];
        es[e_ind] = e_val;
        let value = es
            .iter()
            .enumerate()
            .fold(1u64, |acc, (i, &e)| acc*u64::try_from(PRIMES[i]).unwrap().pow(e.into()));
        Composite{value, es}
    }

    fn set_e(&mut self, ind: usize, new_e: u32) {
        let old_e = self.es[ind];
        if old_e > new_e {
            let change = u64::try_from(PRIMES[ind]).unwrap().pow(old_e - new_e);
            self.value /= change
        } else {
            let change = u64::try_from(PRIMES[ind]).unwrap().pow(new_e - old_e);
            self.value *= change
        }
        self.es[ind] = new_e;
    }

    pub fn try_inc_ind(&mut self, bound: u64, ind: usize) -> bool {
        // try to increment the exponent at index ind and set it to 0 otherwise
        let p = u64::try_from(PRIMES[ind]).unwrap();
        if bound/p < self.value {
            self.set_e(ind, 0);
            return false;
        } else {
            self.value *= p;
            self.es[ind] += 1;
        }
        true
    }

    pub fn inc_vec_with_bound(&mut self, bound: u64) {
        // increment the number represented by the exponents
        for i in 0..self.es.len() {
            if self.try_inc_ind(bound, i) {
                break;
            }
        }
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
