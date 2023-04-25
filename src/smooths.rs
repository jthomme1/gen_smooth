use std::vec::Vec;
use crate::composite::Composite;
use super::{PRIMES};
use std::thread;
use rayon::prelude::*;

struct SmoothsFixedPrimes {
    pub prime_ind: usize,
    pub lower_bound: u128,
    pub upper_bound: u128,
    pub seeds: Vec<Composite>,
}

impl SmoothsFixedPrimes {

    pub fn new(lower_bound: u128, upper_bound: u128, prime_ind: usize) -> Self {
        SmoothsFixedPrimes{
            prime_ind: prime_ind,
            lower_bound: lower_bound,
            upper_bound: upper_bound,
            seeds: vec![]
        }
    }

    pub fn init_gen(&mut self) -> Vec<u128> {
        let ind = self.prime_ind;
        let prime = PRIMES[ind];
        let lower_bound = self.lower_bound;
        let upper_bound = self.upper_bound;
        //println!("{}: Initial generation of smooth numbers", prime);
        // generate all smooth numbers with a fixed exponent for the new prime
        let generate_with_fixed = |e_val: u8| {
            let mut c = Composite::new(ind, e_val);

            let mut new_smooths: Vec<u128> = vec![];
            //let mut new_seeds: Vec<Composite> = vec![];
            let mut add_if_greater = |c: &Composite| {
                // the upper bound is already checked when generating the number
                if lower_bound < c.value {
                    new_smooths.push(c.value);
                }
            };
            add_if_greater(&c);
            // we break if the fixed exponent would change
            loop {
                for i in 0..ind+1 {
                    match c.try_inc_ind(upper_bound, i) {
                        Some(s) => continue,//new_seeds.push(s),
                        None => break,
                    }
                }
                if c.es[ind] == e_val {
                    add_if_greater(&c);
                } else {
                    break
                }
            }
            new_smooths
        };
        // for each possible exponent we start a thread
        let mut rets: Vec<u128> = thread::scope(|s| {
            let mut handles = vec![];
            let p128: u128 = u128::try_from(prime).unwrap();
            let mut p: u128 = p128;
            let mut i = 1;
            while p <= self.upper_bound {
                let h = s.spawn(move || generate_with_fixed(i));
                handles.push(h);
                i += 1;
                p *= p128;
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<Vec<u128>>>().concat()
        });
        rets.par_sort_unstable();
        println!("{}: Generated {} smooth numbers", prime, rets.len());
        rets
    }
/*
    pub fn generate_smooths(&mut self, new_upper_bound: u128) -> Vec<u128> {
        assert!(new_upper_bound >= self.upper_bound);

        let mut rets: (Vec<u128>, Vec<Composite>) = thread::scope(|s| {
            let gen_from_seeds = |i: usize| {
                let mut new_smooths = vec![];
                let mut new_seeds = vec![];
                for si in (i..self.seeds.len()).step_by(*NUM_THREADS) {
                    let s = &self.seeds[si];
                    if s.value > new_upper_bound {
                        new_seeds.push(s.clone());
                        continue;
                    }
                    let mut r = s.clone();
                    let h_ind = r.highest_exp();
                    while r.value >= bef {
                        println!("{}: From seed {s}, produced smooth {}", PRIMES[self.prime_ind], r.value);
                        new_smooths.push(r.value);
                        bef = r.value;
                        for j in 0..h_ind+1 {
                            match r.try_inc_ind(new_upper_bound, j) {
                                Some(n) => {new_seeds.push(n); println!("{}: From seed {s}, produced new seed {n}", PRIMES[self.prime_ind]);},
                                None => break,
                            }
                        }
                    }
                }
                (new_smooths, new_seeds)
            };
            let mut handles = vec![];
            for i in 0..*NUM_THREADS {
                handles.push(s.spawn(move || gen_from_seeds(i)));
            }
            let v: (Vec<Vec<u128>>, Vec<Vec<Composite>>) = handles.into_iter().map(|h| h.join().unwrap()).unzip();
            (v.0.concat(), v.1.concat())
        });

        self.lower_bound = self.upper_bound;
        self.upper_bound = new_upper_bound;
        rets.1.par_sort_unstable();
        println!("{}: Generated {} seeds: {:?}", PRIMES[self.prime_ind], rets.1.len(), rets.1);
        self.seeds = rets.1;
        rets.0.par_sort_unstable();
        println!("{}: Generated {} smooth numbers: {:?}", PRIMES[self.prime_ind], rets.0.len(), rets.0);
        rets.0
    }
    */
}

pub struct Smooths {
    // lower_bound < x <= upper_bound
    pub lower_bound: u128,
    pub upper_bound: u128,
    gens: Vec<SmoothsFixedPrimes>,
    pub smooths: Vec<u128>,
}

impl Smooths {
    pub fn new() -> Self {
        let mut ret = Smooths{lower_bound: 1, upper_bound: 200, gens: vec![], smooths: vec![]};
        // we always already add the 2 smooth numbers
        ret.add_primes(0, 1);
        ret
    }

    pub fn add_primes(&mut self, ind: usize, lower_bound: u128) {
        // if the prime has already been added, do nothing
        if ind+1 <= self.gens.len() {
            return;
        }
        for i in self.gens.len()..ind+1 {
            let mut gen = SmoothsFixedPrimes::new(lower_bound, self.upper_bound, i);
            self.smooths.append(&mut gen.init_gen());
            self.gens.push(gen);
        }
        println!("Sorting all together");
        // sort in parallel
        self.smooths.par_sort_unstable();
        println!("Done sorting all together");
    }

    pub fn get(&self, index: usize) -> Option<u128> {
        if index == self.smooths.len() {
            None
        } else {
            Some(self.smooths[index])
        }
    }

    pub fn next(&mut self, new_upper_bound: u128) {
        self.smooths.clear();
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

