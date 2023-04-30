use crate::smooths::Smooths;
use std::vec::Vec;
use f128::{self, ffi};
use once_cell::sync::Lazy;
use integer_sqrt::IntegerSquareRoot;
use primal;
use std::thread;
use std::cmp::{min, max};
use std::env;

pub mod composite;
pub mod smooths;

// this should suffice for now
static PRIME_BOUND: usize = 2<<20;
static PRIMES: Lazy<Vec<usize>> = Lazy::new(|| primal::Sieve::new(PRIME_BOUND).primes_from(0).collect());
static NUM_THREADS: Lazy<usize> = Lazy::new(|| thread::available_parallelism().unwrap().get());

// we get the logarithm using 128bit floating numbers
fn get_prime_bound(n: u128, c: f64) -> usize {
    max(2, unsafe {
        ffi::powq_f(ffi::log2q_f(f128::f128::new(n)), c.into())
            .try_into()
            .unwrap()
    })
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
    // accessing PRIMES triggers its generataion
    println!("Now generating primes.");
    println!("{} primes generated.", PRIMES.len());
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Provide exactly one argument (upper bound).");
        return;
    }
    let n = u128::from_str_radix(&args[1], 10).unwrap();
    // index to the current smooth number we consider
    let mut smooths = Smooths::new();
    let mut cur: usize = smooths.find_ind_le(2).unwrap();
    let mut c = 1f64;
    // the interval covered by a smooth number (may be off by one because of integer sqrt, but this
    // is not important for the asymptotic behaviour of c)
    let right = |x: u128| {x + 2u128*x.integer_sqrt() + 1u128};
    let left = |x: u128| {x - 2u128*x.integer_sqrt() + 1u128};
    // fn to get the index of the biggest prime below the bound for val and c
    let get_ind = |val: u128, c: f64| {find_highest_prime_ind_below(get_prime_bound(right(val)+1, c))};
    println!("Detected {}-parallelism.", *NUM_THREADS);
    loop {
        // iterate over current range of smooth numbers
        // we go only until the second to last element as there is no gap to consider for the last
        // element
        while cur < smooths.len()-1 {
            let mut ind = get_ind(smooths.get(cur), c);
            // add new smooth numbers
            smooths.add_primes(ind);

            // inner loop for trying to add primes without stretching c
            while cur < smooths.len()-1 {
                // since it is really rare that there is no smooth number in the interval of
                // interest, we parallelize the search
                let step_width: usize = 1 << 20;
                // returns Some(x) if for index x the gap is too big
                let check_gap = |i: usize| -> Option<usize> {
                    let start = min(i*step_width, smooths.len()-1);
                    let stop = min((i+1)*step_width, smooths.len()-1);
                    for x in start..stop {
                        if left(smooths.get(x+1)) > right(smooths.get(x)) {
                            return Some(x);
                        }
                    }
                    None
                };
                let rets: Vec<usize> = thread::scope(|s| {
                    let mut handles = vec![];
                    for i in 0..*NUM_THREADS{
                        let h = s.spawn(move || check_gap(i));
                        handles.push(h);
                    }
                    handles.into_iter().filter_map(|h| h.join().unwrap()).collect()
                });
                match rets.iter().min() {
                    Some(&x) => {
                        cur = x;
                        // the gap was too big, try to add more smooth numbers
                        let new_ind = get_ind(smooths.get(cur), c);
                        if new_ind == ind {
                            // if we were not adding any new smooth numbers, c is too small
                            break;
                        }
                        ind = new_ind;
                        smooths.add_primes(ind);
                    },
                    None => {
                        // advance normally if no gap was found
                        cur = min(cur+*NUM_THREADS*step_width, smooths.len()-1);
                    },
                }
            }
            if cur < smooths.len()-1 {
                // we broke out of the loop without finishing -> c has to be increased
                println!("Gap at {} for c={c}", smooths.get(cur));
                c *= 1.01;
                println!("Setting c={c}");
            }
        }
        let new_upper_bound = min(smooths.upper_bound + smooths.upper_bound/2, n);
        if new_upper_bound == smooths.upper_bound {
            break
        } else {
            println!("Setting upper bound from {} to {}", smooths.upper_bound, new_upper_bound);
            let cur_val = smooths.get(cur);
            smooths.advance(new_upper_bound);
            cur = smooths.find_ind_le(cur_val).unwrap();
            println!("Done setting upper bound");
        }
    }
    println!("Done!");
}
