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
    smooths: Vec<u128>,
}

impl Smooths {
    pub fn new() -> Self {
        let mut ret = Smooths{
            lower_bound: 1,
            upper_bound: 1<<40,
            primes: 0,
            smooths: vec![]
        };
        // we always already add the 2 smooth numbers
        ret.add_primes(0);
        ret
    }

    pub fn len(&self) -> usize {
        self.smooths.len()
    }

    pub fn get(&self, ind: usize) -> u128 {
        self.smooths[ind]>>8
    }

    pub fn add_primes(&mut self, ind: usize) {
        // if the primes have already been added, do nothing
        if ind+1 <= self.primes {
            return;
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
    }

    fn init_gen(&self, ind: usize) -> Vec<u128> {
        let prime = PRIMES[ind];
        let lower_bound = self.lower_bound;
        let upper_bound = self.upper_bound;
        //println!("{}: Initial generation of smooth numbers", prime);
        // generate all smooth numbers with a fixed exponent for the new prime
        let generate_with_fixed = |e_val: u32| {
            let mut c = Composite::new(ind, e_val);

            let mut smooths: Vec<u128> = vec![];
            let mut add_if_greater = |c: &Composite| {
                // the upper bound is already checked when generating the number
                if lower_bound < c.value {
                    // encode the highest prime in the least significant bits
                    // assumes that less that 256 primes are used and the number is less than
                    // 2**120
                    smooths.push((c.value<<8)+u128::try_from(ind).unwrap());
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
            let p128: u128 = u128::try_from(prime).unwrap();
            let mut p: u128 = p128;
            let mut i = 1;
            while p <= upper_bound {
                let h = s.spawn(move || generate_with_fixed(i));
                handles.push(h);
                i += 1;
                p *= p128;
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<Vec<u128>>>().concat()
        });
        smooths.par_sort_unstable();
        println!("{}: Generated {} smooth numbers", prime, smooths.len());
        smooths
    }

    pub fn find_ind_gt(&self, b: u128) -> Option<usize> {
        let shifted = (b<<8)+255;
        if self.smooths.len() == 0 || self.smooths[self.smooths.len()-1] <= shifted {
            return None;
        }
        let ind = match self.smooths.binary_search(&shifted) {
            Ok(x) => x+1,
            Err(x) => x,
        };
        assert!(self.smooths[ind] > shifted);
        Some(ind)
    }

    pub fn find_ind_le(&self, b: u128) -> Option<usize> {
        let shifted = (b<<8)+255;
        if self.smooths.len() == 0 || self.smooths[0] > shifted {
            return None;
        }
        let ind = match self.smooths.binary_search(&shifted) {
            Ok(x) => x,
            Err(x) => x-1,
        };
        assert!(self.smooths[ind] <= shifted);
        Some(ind)
    }

    // we would produce every number exactly once per prime involved. In order to not duplicate the
    // generation, let each number be generated only by the largest prime involved.
    // -> we need to know the largest prime involved.
    // Thus, we store each smooth number with the generator belonging to the highest prime
    // involved.
    // In order to generate the smooth numbers up to the new bound, for each prime p, we need to go
    // over the smooth numbers of the less_or_equal primes. For every prime q, we consider the range
    // (upper_bound/p, new_upper_bound/p) and multiply the numbers in that range by p.
    pub fn advance(&mut self, new_upper_bound: u128) {
        // we need the new upper bound to not be more than two times the old one
        assert!(new_upper_bound/2 <= self.upper_bound);
        assert!(new_upper_bound > self.upper_bound);
        let produce = |low_ind: usize, high_ind: usize, prime_ind: usize| -> Vec<u128> {
            let mut ret: Vec<u128> = vec![];
            let p = u128::try_from(PRIMES[prime_ind]).unwrap();
            for i in low_ind..high_ind+1 {
                let n = self.smooths[i];
                if usize::try_from(n&255).unwrap() <= prime_ind {
                    ret.push((((n>>8)*p)<<8)+u128::try_from(prime_ind).unwrap());
                }
            }
            ret
        };
        let mut smooths = thread::scope(|s| {
            let mut handles = vec![];
            for i in 0..self.primes {
                let p = u128::try_from(PRIMES[i]).unwrap();
                let lower = self.upper_bound/p;
                let upper = new_upper_bound/p;
                let low_ind = self.find_ind_gt(lower).unwrap();
                let high_ind = self.find_ind_le(upper).unwrap();
                let h = s.spawn(move || produce(low_ind, high_ind, i));
                handles.push(h);
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<Vec<u128>>>().concat()
        });
        smooths.par_sort_unstable();
        let mut lower_bound_factor = u128::try_from(PRIMES[self.primes-1]).unwrap();
        lower_bound_factor += lower_bound_factor/6; 
        self.lower_bound = new_upper_bound/lower_bound_factor;
        self.upper_bound = new_upper_bound;
        let ind = self.find_ind_le(self.lower_bound).unwrap()+1;
        let new_len = self.smooths.len() - ind;
        let old_len = self.smooths.len();
        self.smooths.copy_within(ind..old_len, 0);
        self.smooths.truncate(new_len);
        self.smooths.append(&mut smooths);
    }
}

