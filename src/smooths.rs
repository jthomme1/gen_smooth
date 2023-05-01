use std::vec::Vec;
use crate::composite::Composite;
use super::{PRIMES};
use std::thread;
use rayon::prelude::*;

pub struct Smooths {
    // lower_bound <= x <= upper_bound
    pub lower_bound: u64,
    pub upper_bound: u64,
    primes: usize,
    smooths: Vec<u64>,
}

impl Smooths {
    pub fn new(n: u64) -> Self {
        let mut ret = Smooths{
            lower_bound: 1,
            upper_bound: n,
            primes: 0,
            smooths: vec![]
        };
        // we always already add the 2 smooth numbers
        ret.add_primes(0, None);
        ret
    }

    pub fn len(&self) -> usize {
        self.smooths.len()
    }

    pub fn get(&self, ind: usize) -> u64 {
        self.smooths[ind]
    }

    pub fn ind(&self) -> usize {
        self.primes
    }

    pub fn add_primes(&mut self, ind: usize, cur: Option<usize>) -> bool {
        // if the primes have already been added, do nothing
        if ind+1 <= self.primes {
            return false;
        }
        if let Some(c) = cur {
            self.lower_bound = self.smooths[c];
            let old_len = self.smooths.len();
            self.smooths.copy_within(c..old_len, 0);
            self.smooths.truncate(old_len-c);
        }
        for i in self.primes..ind+1 {
            let mut smooths = self.init_gen(i);
            self.smooths.append(&mut smooths);
        }
        self.primes = ind+1;
        println!("Sorting all together");
        // sort in parallel
        self.smooths.par_sort_unstable();
        println!("Done adding primes");
        true
    }

    pub fn find_ind_gt(&self, b: u64) -> Option<usize> {
        if self.smooths.len() == 0 || self.smooths[self.smooths.len()-1] <= b {
            return None;
        }
        let ind = match self.smooths.binary_search(&b) {
            Ok(x) => x+1,
            Err(x) => x,
        };
        assert!(self.smooths[ind] > b);
        Some(ind)
    }

    pub fn find_ind_le(&self, b: u64) -> Option<usize> {
        if self.smooths.len() == 0 || self.smooths[0] > b {
            return None;
        }
        let ind = match self.smooths.binary_search(&b) {
            Ok(x) => x,
            Err(x) => x-1,
        };
        assert!(self.smooths[ind] <= b);
        Some(ind)
    }

    fn init_gen(&self, ind: usize) -> Vec<u64> {
        let prime = PRIMES[ind];
        let lower_bound = self.lower_bound;
        let upper_bound = self.upper_bound;
        //println!("{}: Initial generation of smooth numbers", prime);
        // generate all smooth numbers with a fixed exponent for the new prime
        let generate_with_fixed = |e_val: u32| {
            let mut c = Composite::new(ind, e_val);

            let mut smooths: Vec<u64> = vec![];
            let mut add_if_greater = |c: &Composite| {
                // the upper bound is already checked when generating the number
                if lower_bound <= c.value {
                    smooths.push(c.value);
                }
            };
            add_if_greater(&c);
            // we break if the fixed exponent would change
            loop {
                c.inc_vec_with_bound(upper_bound);
                if c.es[ind] == e_val {
                    add_if_greater(&c);
                } else {
                    break
                }
            }
            smooths
        };
        // for each possible exponent we start a thread
        let mut smooths = thread::scope(|s| {
            let mut handles = vec![];
            let p64: u64 = u64::try_from(prime).unwrap();
            let mut p: u64 = p64;
            let mut i = 1;
            loop {
                let h = s.spawn(move || generate_with_fixed(i));
                handles.push(h);
                i += 1;
                // to avoid overflow
                if u64::MAX/p64 < p {
                    break;
                }
                p *= p64;
                if p > upper_bound {
                    break;
                }
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<Vec<u64>>>().concat()
        });
        smooths.par_sort_unstable();
        println!("{}: Generated {} smooth numbers", prime, smooths.len());
        smooths
    }
}

