use std::vec::Vec;
use std::cmp::min;
use crate::composite::Composite;
use super::{PRIMES};
use std::thread;
use rayon::prelude::*;
use std::io::{self, Write};

pub struct Smooths {
    // lower_bound < x <= upper_bound
    pub lower_bound: u64,
    pub upper_bound: u64,
    primes: usize,
    smooths: Vec<u64>,
}

impl Smooths {
    pub fn new(bound: u64) -> Self {
        let mut ret = Smooths{
            lower_bound: 1,
            upper_bound: min(1<<40, bound),
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

    pub fn get(&self, ind: usize) -> u64 {
        self.smooths[ind]
    }

    pub fn ind(&self) -> usize {
        self.primes
    }

    pub fn print_smooths(&self) {
        print!("[");
        for i in self.smooths.iter() {
            print!("{}, ", i);
        }
        print!("]\n");
        io::stdout().flush().unwrap();
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

    fn init_gen(&self, ind: usize) -> Vec<u64> {
        let prime = PRIMES[ind];
        let lower_bound = self.lower_bound;
        let upper_bound = self.upper_bound;
        // generate all smooth numbers with a fixed exponent for the new prime
        let generate_with_fixed = |e_val: u32| {
            let mut c = Composite::new(ind, e_val);

            let mut smooths: Vec<u64> = vec![];
            let mut add_if_greater = |c: &Composite| {
                // the upper bound is already checked when generating the number
                if lower_bound < c.value {
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

    fn highest_prime_ind_involved(&self, n: u64) -> usize {
        for i in (0..self.primes).rev() {
            let p = u64::try_from(PRIMES[i]).unwrap();
            if n % p == 0 {
                return i;
            }
        }
        panic!("not-smooth number found");
    }

    // we would produce every number exactly once per prime involved. In order to not duplicate the
    // generation, let each number be generated only by the largest prime involved.
    // -> we need to know the largest prime involved.
    // Thus, we store each smooth number with the generator belonging to the highest prime
    // involved.
    // In order to generate the smooth numbers up to the new bound, for each prime p, we need to go
    // over the smooth numbers of the less_or_equal primes. For every prime q, we consider the range
    // (upper_bound/p, new_upper_bound/p) and multiply the numbers in that range by p.
    pub fn advance(&mut self, new_upper_bound: u64) {
        // we need the new upper bound to not be more than two times the old one
        assert!(new_upper_bound/2 <= self.upper_bound);
        assert!(new_upper_bound > self.upper_bound);
        let produce = |low_ind: usize, high_ind: usize, prime_ind: usize| -> Vec<u64> {
            let mut ret: Vec<u64> = vec![];
            let p = u64::try_from(PRIMES[prime_ind]).unwrap();
            for i in low_ind..high_ind+1 {
                let n = self.smooths[i];
                if u64::MAX/p >= n && self.highest_prime_ind_involved(n) <= prime_ind {
                    ret.push(n*p);
                }
            }
            ret
        };
        let mut smooths = thread::scope(|s| {
            let mut handles = vec![];
            for i in 0..self.primes {
                let p = u64::try_from(PRIMES[i]).unwrap();
                let lower = self.upper_bound/p;
                let upper = new_upper_bound/p;
                let low_ind = self.find_ind_le(lower).unwrap()+1;
                let high_ind = self.find_ind_le(upper).unwrap();
                let h = s.spawn(move || produce(low_ind, high_ind, i));
                handles.push(h);
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<Vec<u64>>>().concat()
        });
        smooths.par_sort_unstable();
        let lower_bound_factor = u64::try_from(PRIMES[self.primes+3]).unwrap();
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

