use std::cmp::{PartialEq, PartialOrd, Ord, Ordering, Eq};
use std::vec::Vec;
use super::PRIMES;

#[derive(Eq, Debug)]
pub struct Composite {
    pub value: u128,
    pub es: Vec<u32>,
}

impl Composite {
    pub fn new(e_ind: usize, e_val: u32) -> Self {
        let mut es: Vec<u32> = vec![0; e_ind+1];
        es[e_ind] = e_val;
        let value = es
            .iter()
            .enumerate()
            .fold(1u128, |acc, (i, &e)| acc*(PRIMES[i]).pow(e.into()));
        Composite{value, es}
    }

    fn set_e(&mut self, ind: usize, new_e: u32) {
        let old_e = self.es[ind];
        if old_e > new_e {
            let change = PRIMES[ind].pow(old_e - new_e);
            self.value /= change
        } else {
            let change = PRIMES[ind].pow(new_e - old_e);
            self.value *= change
        }
        self.es[ind] = new_e;
    }

    pub fn try_inc_ind(&mut self, bound: u128, ind: usize) -> bool {
        // try to increment the exponent at index ind and set it to 0 otherwise
        let p = PRIMES[ind];
        if bound/p < self.value {
            self.set_e(ind, 0);
            return false;
        } else {
            self.value *= p;
            self.es[ind] += 1;
        }
        true
    }

    pub fn inc_vec_with_bound(&mut self, bound: u128) -> bool {
        // increment the number represented by the exponents
        for i in 0..self.es.len() {
            if self.try_inc_ind(bound, i) {
                return true;
            }
        }
        false
    }

    pub fn try_inc_by_n(&mut self, n: u32, bound: u128) -> u32 {
        // try to increment the exponent at index ind by n and set it to 0 otherwise
        let factor_left = bound/self.value;
        let max_advance = factor_left.ilog2();
        if max_advance >= n {
            self.value *= 1<<n;
            self.es[0] += n;
            return 0;
        } else {
            self.value *= 1<<max_advance;
            self.es[0] += max_advance;
            return n-max_advance;
        }
    }

    pub fn inc_vec_by_n_with_bound(&mut self, n: usize, bound: u128) -> bool {
        /*
        for _ in 0..n {
            if !self.inc_vec_with_bound(bound) {
                return false;
            }
        }
        */

        let mut left = u32::try_from(n).unwrap();
        // we try to increment by multiplying by 2**n. If this is to big, take the biggest power of
        // 2 that is possible and then increment normally before repeating until we incremented n
        // times. In the worst case, we increment in single steps.
        while left > 0 {
            left = self.try_inc_by_n(left, bound);
            if left > 0 {
                if !self.inc_vec_with_bound(bound) {
                    return false;
                }
                left -= 1;
            }
        }
        true
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
