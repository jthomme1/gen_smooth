use crate::composite::Composite;
use std::vec::Vec;
use f128::{self, ffi};
use once_cell::sync::Lazy;
use primal;

pub mod composite;

static PRIME_BOUND: usize = 2<<26;
static PRIMES: Lazy<Vec<usize>> = Lazy::new(|| primal::Sieve::new(PRIME_BOUND).primes_from(0).collect());

struct Smooths {
    bound: u128,
    index: Option<usize>,
    nr_primes: usize,
    smooths: Vec<u128>,
}

impl Smooths {
    fn upto(bound: u128) -> Self {
        Smooths{bound: bound, index: None, nr_primes: 0, smooths: vec![]}
    }

    fn add_prime(&mut self, single: bool) {
        let val_prev = self.index.and_then(|i| Some(self.smooths[i]));
        let prime = PRIMES[self.nr_primes];
        assert!(u128::try_from(prime).unwrap() < self.bound);
        println!("{}: generating smooth numbers", prime);
        self.nr_primes += 1;
        let es: Vec<u32> = vec![0; self.nr_primes];

        let mut new_smooths: Vec<u128> = vec![];
        let mut c = Composite::new(es);
        while c.inc_vec_with_bound(self.bound) {
            match val_prev {
                Some(val_p) if val_p < c.value => {
                    new_smooths.push(c.value);
                },
                None => new_smooths.push(c.value),
                _ => (),
            };
        }
        println!("{}: Generated {} smooth numbers", prime, new_smooths.len());
        println!("{}: Now sorting", prime);
        new_smooths.sort();
        println!("{}: Done sorting", prime);
        self.smooths.append(&mut new_smooths);
        self.index = None;
        if single {
            self.smooths.sort();
            self.index = val_prev.and_then(|val| Some(self.get_ind(val)));
        }
    }

    fn add_primes_upto_ind(&mut self, ind: usize) {
        let val_prev = self.index.and_then(|i| Some(self.smooths[i]));
        for _ in self.nr_primes..ind+1 {
            self.add_prime(false);
        }
        println!("Sorting all together");
        self.smooths.sort();
        self.index = val_prev.and_then(|val| Some(self.get_ind(val)));
        println!("Done sorting all together");
    }

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

    fn next(&mut self) -> Option<u128> {
        match self.index {
            Some(i) if i == self.smooths.len()-1 => None,
            Some(i) => {
                self.index = Some(i+1);
                Some(self.smooths[i+1])
            },
            None => {
                self.index = Some(0);
                Some(self.smooths[0])
            }
        }
    }
}

fn sqrt_floor(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut s: u128 = unsafe{ffi::sqrtq_f(f128::f128::new(n)).try_into().unwrap()};
    while s*s > n {
        s -= 1;
    }
    while (s+1)*(s+1) <= n {
        s += 1;
    }
    s
}

fn sqrt_ceil(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut s: u128 = unsafe{ffi::sqrtq_f(f128::f128::new(n)).try_into().unwrap()};
    while s*s < n {
        s += 1;
    }
    while (s-1)*(s-1) >= n {
        s -= 1;
    }
    s
}

fn get_prime_bound(n: u128, c: f64) -> usize {
    unsafe {
        ffi::powq_f(ffi::log2q_f(f128::f128::new(n)), c.into())
            .try_into()
            .unwrap()
    }
}

fn find_highest_prime_ind_below(u: usize) -> usize {
    let mut l: usize = 0;
    let mut r: usize = PRIMES.len();
    while l + 1 < r {
        let m = l + (r-l)/2;
        if PRIMES[m] > u {
            r = m;
        } else {
            l = m;
        }
    }
    assert!(PRIMES[l] <= u);
    assert!(PRIMES[l+1] > u);
    l
}


fn main() {
    println!("Now generating primes.");
    println!("{} primes generated.", PRIMES.len());
    let n = u128::from_str_radix("1237940039285380274899124224", 10).unwrap();
    let mut smooths = Smooths::upto(n);
    smooths.add_primes_upto_ind(0);
    let mut cur = 1u128;
    let mut c = 1f64;
    let right = |x: u128| {x + 2u128*sqrt_ceil(x) + 1u128};
    let left = |x: u128| {x - 2u128*sqrt_floor(x) + 1u128};
    let get_ind = |val: u128, c: f64| {find_highest_prime_ind_below(get_prime_bound(right(val)+1, c))};
    while c < 4.0 && cur <= n {
        let mut ind = get_ind(cur, c);
        loop {
            smooths.add_primes_upto_ind(ind);
            let mut broke = false;
            while let Some(next) = smooths.next() {
                if left(next) <= right(cur) {
                    cur = next;
                } else {
                    broke = true;
                    println!("Breaking at cur: {cur} because left({next}) > right({cur}) ({} > {})", left(next), right(cur));
                    cur = next;
                    break;
                }
            }
            if broke {
                // we got here not because we were done, but because the gap was too big
                let new_ind = get_ind(cur, c);
                if new_ind == ind {
                    break;
                }
                ind += 1;
            }
        }
        c *= 1.01;
        println!("cur: {}, c: {}", cur, c);
    }
    /*
     * tactics:
     * 1. generate primes up to a number big enough
     * 2. iteratively generate smooth numbers corresponding to bound (can be influenced by c and not by n (fixed))
     * 3. Check the intervals. For each number to be checked, the bounds on u change.
     * 4. generate plot c values and n
     */
}
//TODO: avoid sorting, double check correctness, use threads
