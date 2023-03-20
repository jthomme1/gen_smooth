use std::vec::Vec;
use crate::composite::Composite;
use super::PRIMES;
use std::thread;
use closure::closure;
use rayon::prelude::*;

pub struct Smooths {
    pub bound: u128,
    pub nr_primes: usize,
    pub smooths: Vec<u128>,
}

impl Smooths {
    pub fn upto(bound: u128, cur: &mut usize) -> Self {
        assert!(bound > 2);
        let mut ret = Smooths{bound: bound, nr_primes: 0, smooths: vec![]};
        ret.add_primes_and_cut(0, cur);
        ret
    }

    fn gen_smooths_for_prime(bound: u128, ind: usize, lower_bound: u128) -> Vec<u128> {
        let prime = PRIMES[ind];
        assert!(u128::try_from(prime).unwrap() < bound);
        println!("{}: Generating smooth numbers", prime);
        let generate_with_fixed = move |e_val: u32| {
            let mut es: Vec<u32> = vec![0; ind+1];
            es[ind] = e_val;

            let mut new_smooths: Vec<u128> = vec![];
            let mut c = Composite::new(es);
            let mut add_if_greater = |c: &Composite| {
                if lower_bound < c.value {
                    new_smooths.push(c.value);
                }
            };
            add_if_greater(&c);
            while c.inc_vec_with_bound(bound) && c.es[ind] == e_val {
                add_if_greater(&c);
            }
            new_smooths
        };
        let mut rets: Vec<u128> = thread::scope(|s| {
            let mut handles = vec![];
            let p128: u128 = u128::try_from(prime).unwrap();
            let mut p: u128 = p128;
            let mut i = 0;
            while p <= bound {
                let h = s.spawn(move || generate_with_fixed(i));
                handles.push(h);
                i += 1;
                p *= p128;
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect::<Vec<Vec<u128>>>().concat()
        });
        println!("{}: Generated {} smooth numbers", prime, rets.len());
        println!("{}: Now sorting", prime);
        rets.par_sort_unstable();
        println!("{}: Done sorting", prime);
        rets
    }

    pub fn add_primes_and_cut(&mut self, ind: usize, cur: &mut usize) {
        self.smooths = self.smooths.split_off(*cur);
        *cur = 0;
        if ind+1 <= self.nr_primes {
            return;
        }
        let val_prev: u128 = if self.nr_primes == 0 {0} else {self.smooths[0]};
        let mut rets: Vec<Vec<u128>> = thread::scope(|s| {
            let mut handles = vec![];
            for i in self.nr_primes..ind+1 {
                let bound = self.bound;
                let h = s.spawn(closure!(move bound, move i, ref val_prev, || {
                    Self::gen_smooths_for_prime(bound, i, *val_prev)
                }));
                handles.push(h);
            }
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        rets.iter_mut().for_each(|r| self.smooths.append(r));
        println!("Sorting all together");
        self.smooths.par_sort_unstable();
        println!("Done sorting all together");
        assert!(self.smooths[0] == val_prev || val_prev == 0);
        self.nr_primes = ind+1;
    }

    /*
    fn get_ind(&self, val: u128) -> usize {
        let mut l: usize = 0;
        let mut r: usize = self.smooths.len();
        while l + 1 < r {
            let m = l + (r-l)/2;
            if self.smooths[m] > val {
                r = m;
            } else {
                l = m;
            }
        }
        assert!(val == self.smooths[l]);
        l
    }
    */
    
    pub fn get(&self, index: usize) -> Option<u128> {
        if index == self.smooths.len() {
            None
        } else {
            Some(self.smooths[index])
        }
    }
}

