use std::vec::Vec;
use crate::composite::Composite;
use super::{PRIMES};
use std::thread;
use rayon::prelude::*;

pub struct Smooths {
    // lower_bound < x <= upper_bound
    pub lower_bound: u128,
    pub upper_bound: u128,
    primes: usize,
    pub small_smooths: Vec<u64>,
    pub big_smooths: Vec<u128>,
}

impl Smooths {
    pub fn new() -> Self {
        let mut ret = Smooths{
            lower_bound: 1,
            upper_bound: 1<<40,
            primes: 0,
            small_smooths: vec![],
            big_smooths: vec![]
        };
        // we always already add the 2 smooth numbers
        ret.add_primes(0);
        ret
    }

    pub fn add_primes(&mut self, ind: usize) {
        // if the primes have already been added, do nothing
        if ind+1 <= self.primes {
            return;
        }
        for i in self.primes..ind+1 {
            let (small, big) = self.init_gen(i);
            self.small_smooths.append(small);
            self.big_smooths.append(big);
        }
        println!("Sorting all together");
        // sort in parallel
        self.small_smooths.par_sort_unstable();
        self.big_smooths.par_sort_unstable();
        println!("Done adding primes");
    }

    pub fn init_gen(&self, ind: usize) -> (Vec<u64>, Vec<u128>) {
        let prime = PRIMES[ind];
        let lower_bound = self.lower_bound;
        let upper_bound = self.upper_bound;
        //println!("{}: Initial generation of smooth numbers", prime);
        // generate all smooth numbers with a fixed exponent for the new prime
        let generate_with_fixed = |e_val: u8| {
            let mut c = Composite::new(ind, e_val);

            let mut new_small: Vec<u64> = vec![];
            let mut new_big: Vec<u128> = vec![];
            let mut add_if_greater = |c: &Composite| {
                // the upper bound is already checked when generating the number
                if lower_bound < c.value {
                    if c.value <= u128::from(u64::MAX) {
                        new_small.push(u64::try_from(c.value).unwrap());
                    } else {
                        new_big.push(c.value);
                    }
                }
            };
            add_if_greater(&c);
            // we break if the fixed exponent would change
            loop {
                c.inc_vec_with_bound(upper_bound, ind);
                if c.es[ind] == e_val {
                    add_if_greater(&c);
                } else {
                    break
                }
            }
            (new_small, new_big)
        };
        // for each possible exponent we start a thread
        let mut (small, big) = thread::scope(|s| {
            let mut handles = vec![];
            let p128: u128 = u128::try_from(prime).unwrap();
            let mut p: u128 = p128;
            let mut i = 1;
            while p <= upper_bound {
                let h = s.spawn(move || generate_with_fixed(i));
                handles.push(h);
                i += 1;
                p *= p128;
            }
            let v: (Vec<Vec<u64>>, Vec<Vec<u128>>) = handles.into_iter().map(|h| h.join().unwrap()).unzip();
            (v.0.concat(), v.1.concat())
        });
        small.par_sort_unstable();
        big.par_sort_unstable();
        println!("{}: Generated {} small smooth numbers", prime, small.len());
        println!("{}: Generated {} big smooth numbers", prime, big.len());
        (small, big)
    }

    pub fn find_big_ind_gt(&self, b: u128) -> Option<usize> {
        if self.big.len() == 0 || self.big[self.big.len()-1] <= b {
            return None;
        }
        let ind = match self.big.binary_search(b) {
            Ok(x) => x+1,
            Err(x) => x,
        }
        Some(ind)
    }

    pub fn find_small_ind_gt(&self, big_b: u128) -> Option<usize> {
        if self.small.len() == 0 || self.small[self.small.len()-1] <= big_b {
            return None;
        }
        let b = match u64::try_from(big_b) {
            Ok(b) => b,
            Err(_) => return None,
        };
        let ind = match self.small.binary_search(b) {
            Ok(x) => x+1,
            Err(x) => x,
        }
        Some(ind)
    }

    pub fn find_big_ind_le(&self, b: u128) -> Option<usize> {
        if self.big.len() == 0 || self.big[0] > b {
            return None;
        }
        let ind = match self.big.binary_search(b) {
            Ok(x) => x,
            Err(x) => x-1,
        }
        Some(ind)
    }

    pub fn find_small_ind_le(&self, big_b: u128) -> Option<usize> {
        if self.small.len() == 0 || self.small[0] > big_b {
            return None;
        }
        let b = match u64::try_from(big_b) {
            Ok(b) => b,
            Err(_) => return Some(self.small.len()-1),
        };
        let ind = match self.small.binary_search(b) {
            Ok(x) => x+1,
            Err(x) => x,
        }
        Some(ind)
    }

    pub fn advance_small(&self, low_ind: usize, high_ind: usize) -> (Vec<u64>, Vec<u128) {
    }

    // we would produce every number exactly once per prime involved. In order to not duplicate the
    // generation, let each number be generated only by the largest prime involved.
    // -> we need to know the largest prime involved.
    // Thus, we store each smooth number with the generator belonging to the highest prime
    // involved.
    // In order to generate the smooth numbers up to the new bound, for each prime p, we need to go
    // over the smooth numbers of the less_or_equal primes. For every prime q, we consider the range
    // (upper_bound/p, new_upper_bound/p) and multiply the numbers in that range by p.
    pub fn advance_big(&self, low_ind: usize, high_ind: usize) -> (Vec<u64>, Vec<u128) {
    }

    pub fn advance(&mut self, new_upper_bound: u128) {
        // we need the new upper bound to not be more than two times the old one
        assert!(new_upper_bound/2 <= self.upper_bound);
        // get the elements of big and small in range: upper_bound/p until new_upper_bound/p
        thread::scope(|s| {
            let mut handles = vec![];
            for i in 0..self.primes {
                let p = PRIMES[i];
                let lower = self.upper_bound/p;
                let upper = new_upper_bound/p;
                // small: 3 cases.
                //   1: lower and upper both inside
                //   2: only lower inside
                //   3: none inside
                if let Some(low_ind) = self.find_small_ind_gt(lower) {
                    let high_ind = self.find_small_ind_le(upper).unwrap();
                    let h = s.spawn(|| something_small(low_ind, high_ind));
                    handles.push(h);
                }
                // big: 3 cases.
                //   1: lower and upper both inside
                //   2: only upper inside
                //   3: none inside
                if let Some(high_ind) = self.find_big_ind_le(upper) {
                    let low_ind = self.find_big_ind_gt(lower).unwrap();
                    let h = s.spawn(|| something_big(low_ind, high_ind));
                    handles.push(h);
                }
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<Vec<u128>>>().concat()
        });
        find_first_big_ind_gt(self.upper_bound/p)
        // construct the new elements
        // merge and sort them
        // add them to small and big
        // retain last element
        self.smooths = vec![self.smooths[self.smooths.len()-1]];
        self.smooths = thread::scope(|s| {
            let mut handles = vec![];
            for g in self.gens.iter_mut() {
                let h = s.spawn(move || { g.lower_bound = g.upper_bound; g.upper_bound = new_upper_bound; g.init_gen()});
                handles.push(h);
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<Vec<u128>>>().concat()
        });
        self.smooths.par_sort_unstable();
        self.lower_bound = self.upper_bound;
        self.upper_bound = new_upper_bound;
    }
}

