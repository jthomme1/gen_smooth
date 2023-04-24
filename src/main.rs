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

// this should suffice for now
static PRIME_BOUND: usize = 2<<20;
static PRIMES: Lazy<Vec<usize>> = Lazy::new(|| primal::Sieve::new(PRIME_BOUND).primes_from(0).collect());
static NUM_THREADS: Lazy<usize> = Lazy::new(|| thread::available_parallelism().unwrap().get());

// we get the logarithm using 128bit floating numbers
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
    let mut cur: usize = 0;
    let mut smooths = Smooths::new();
    let mut c = 1f64;
    // the interval covered by a smooth number (may be off by one because of integer sqrt, but this
    // is not important for the asymptotic behaviour of c)
    let right = |x: u128| {x + 2u128*x.integer_sqrt() + 1u128};
    let left = |x: u128| {x - 2u128*x.integer_sqrt() + 1u128};
    // fn to get the index of the biggest prime below the bound for val and c
    let get_ind = |val: u128, c: f64| {find_highest_prime_ind_below(get_prime_bound(right(val)+1, c))};
    let num_threads = thread::available_parallelism().unwrap().get();
    println!("Detected {num_threads}-parallelism.");
    while smooths.lower_bound < n {
        //println!("Start of outermost loop - not reached {n} yet");
        // iterate over current range of smooth numbers
        while cur < smooths.smooths.len() {
            let cur_val = smooths.smooths[cur];
            //println!("Still iterating through current smooth numbers ({cur_val}) {cur} < {}", smooths.smooths.len());
            let mut ind = get_ind(cur_val, c);
            // add new smooth numbers
            smooths.add_primes(ind, cur_val);

            // inner loop for trying to add primes without stretching c
            while cur < smooths.smooths.len() {
                // since it is really rare that there is no smooth number in the interval of
                // interest, we parallelize the search
                let step_width: usize = 1 << 20;
                // returns Some(x) if for index x the gap is too big
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
                    for i in 0..num_threads {
                        let h = s.spawn(move || do_part(i));
                        handles.push(h);
                    }
                    handles.into_iter().filter_map(|h| h.join().unwrap()).collect()
                });
                match rets.iter().min() {
                    Some(&x) => {
                        // the gap was too big, try to add more smooth numbers
                        let new_ind = get_ind(smooths.smooths[x], c);
                        if new_ind == ind {
                            // if we were not adding any new smooth numbers, c is too small
                            break;
                        }
                        ind = new_ind;
                        cur = x;
                        smooths.add_primes(ind, cur_val);
                    },
                    None => {
                        // advance normally if no gap was found
                        cur = min(cur+num_threads*step_width, smooths.smooths.len());
                    },
                }
            }
            if cur < smooths.smooths.len() {
                // we broke out of the loop without finishing -> c has to be increased
                println!("Gap at {} for c={c}", smooths.smooths[cur]);
                c *= 1.01;
                println!("Setting c={c}");
            }
        }
        let new_upper_bound = min(smooths.upper_bound + (smooths.upper_bound>>1), n);
        println!("Setting upper bound from {} to {}", smooths.upper_bound, new_upper_bound);
        smooths.next(new_upper_bound);
        println!("Done setting upper bound");
    }
    println!("Done!");
}
//TODO: avoid sorting + storing, other technique?
