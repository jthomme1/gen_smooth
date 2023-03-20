use crate::smooths::Smooths;
use std::vec::Vec;
use f128::{self, ffi};
use once_cell::sync::Lazy;
use integer_sqrt::IntegerSquareRoot;
use primal;
use std::thread;
use std::cmp::min;
use std::env;

pub mod composite;
pub mod smooths;

static PRIME_BOUND: usize = 2<<20;
static PRIMES: Lazy<Vec<usize>> = Lazy::new(|| primal::Sieve::new(PRIME_BOUND).primes_from(0).collect());
static NUM_THREADS: usize = 8;

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
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2, "Provide exactly one argument (upper bound).");
    let n = u128::from_str_radix(&args[1], 10).unwrap();
    let mut cur: usize = 0;
    let mut smooths = Smooths::upto(n, &mut cur);
    let mut c = 1f64;
    let right = |x: u128| {x + 2u128*x.integer_sqrt() + 1u128};
    let left = |x: u128| {x - 2u128*x.integer_sqrt() + 1u128};
    let get_ind = |val: u128, c: f64| {find_highest_prime_ind_below(get_prime_bound(right(val)+1, c))};
    while cur < smooths.smooths.len()-1 && c < 4.0 {
        let cur_val = smooths.smooths[cur];
        let mut ind = get_ind(cur_val, c);
        smooths.add_primes_and_cut(ind, &mut cur);

        // inner loop for trying to add primes without stretching c
        while cur < smooths.smooths.len()-1 {
            let step_width: usize = 1 << 20;
            let do_part = |i: usize| -> Option<usize> {
                let start = min(i*step_width, smooths.smooths.len()-1);
                let stop = min((i+1)*step_width, smooths.smooths.len()-1);
                for x in start..stop {
                    if left(smooths.smooths[x+1]) > right(smooths.smooths[x]) {
                        return Some(x);
                    }
                }
                None
            };
            let rets: Vec<usize> = thread::scope(|s| {
                let mut handles = vec![];
                for i in 0..NUM_THREADS {
                    let h = s.spawn(move || do_part(i));
                    handles.push(h);
                }
                handles.into_iter().filter_map(|h| h.join().unwrap()).collect()
            });
            match rets.iter().min() {
                Some(&x) => {
                    let new_ind = get_ind(smooths.smooths[x], c);
                    if new_ind == ind {
                        break;
                    }
                    ind = new_ind;
                    cur = x;
                    smooths.add_primes_and_cut(ind, &mut cur);
                },
                None => {
                    cur = min(cur+NUM_THREADS*step_width, smooths.smooths.len()-1);
                },
            }
        }
        if cur < smooths.smooths.len()-1 {
            println!("Gap at {} for c={c}", smooths.smooths[cur]);
            c *= 1.01;
            println!("Setting c={c}");
        }
    }
    println!("Done at {}", smooths.smooths[cur]);
    /*
     * tactics:
     * 1. generate primes up to a number big enough
     * 2. iteratively generate smooth numbers corresponding to bound (can be influenced by c and not by n (fixed))
     * 3. Check the intervals. For each number to be checked, the bounds on u change.
     * 4. generate plot c values and n
     */
}
//TODO: avoid sorting + storing, parallelize generation, other technique?
