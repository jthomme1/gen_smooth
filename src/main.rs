use crate::smooths::Smooths;
use std::vec::Vec;
use once_cell::sync::Lazy;
use primal;
use std::thread;
use std::cmp::min;
use std::env;
use std::str::FromStr;

pub mod composite;
pub mod smooths;

// this should suffice for now
static PRIME_BOUND: usize = 2<<20;
static PRIMES: Lazy<Vec<usize>> = Lazy::new(|| primal::Sieve::new(PRIME_BOUND).primes_from(0).collect());
static NUM_THREADS: Lazy<usize> = Lazy::new(|| thread::available_parallelism().unwrap().get());

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        println!("Provide exactly three arguments (upper bound, whether log (0) or exp (1), exponent).");
        return;
    }
    let n = u64::from_str_radix(&args[1], 10).unwrap();
    let exp = u64::from_str_radix(&args[2], 10).unwrap();
    let e = f64::from_str(&args[3]).unwrap();
    println!("{n}, {exp}, {e}");
    // index to the current smooth number we consider
    let mut smooths = Smooths::new(n);
    let mut cur: usize = smooths.find_ind_le(2).unwrap();
    // the interval covered by a smooth number
    let width = |x: u64| {
        if exp == 1 {
            (x as f64).powf(e) as u64
        } else {
            (x as f64).log2().powf(e) as u64
        }
    };
    let right = |x: u64| {
        if u64::MAX - width(x) - 1u64 < x {
            return u64::MAX;
        }
        x + width(x) + 1u64
    };
    let left = |x: u64| {
        if width(x) - 1u64 > x {
            return 0
        }
        x - width(x) + 1u64
    };
    // fn to get the index of the biggest prime below the bound for val and c
    println!("Detected {}-parallelism.", *NUM_THREADS);
    // iterate over current range of smooth numbers
    // we go only until the second to last element as there is no gap to consider for the last
    // element
    while right(smooths.get(cur)) < n {
        if cur == smooths.len() - 1 {
            if right(smooths.get(cur)) >= smooths.upper_bound {
                let new_upper_bound = {
                    if u64::MAX - smooths.upper_bound/2 < smooths.upper_bound {
                        u64::MAX
                    } else {
                        min(smooths.upper_bound + smooths.upper_bound/2, n)
                    }
                };
                println!("Setting upper bound from {} to {}", smooths.upper_bound, new_upper_bound);
                let cur_val = smooths.get(cur);
                smooths.advance(new_upper_bound);
                cur = smooths.find_ind_le(cur_val).unwrap();
                println!("Done setting upper bound");
            } else {
                let cur_val = smooths.get(cur);
                println!("Gap at {cur_val} with primes up to {}", PRIMES[smooths.primes-1]);
                smooths.add_prime();
                cur = smooths.find_ind_le(cur_val).unwrap();
            }
        } else {
            // since it is really rare that there is no smooth number in the interval of
            // interest, we parallelize the search
            let step_width: usize = 1 << 14;
            // returns Some(x) if for index x the gap is too big
            let check_gap = |i: usize| -> Option<usize> {
                let start = min(cur+i*step_width, smooths.len()-1);
                let stop = min(cur+(i+1)*step_width, smooths.len()-1);
                for x in start..stop {
                    if left(smooths.get(x+1)) > right(smooths.get(x)) + 1 {
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
                    let cur_val = smooths.get(cur);
                    println!("Gap between {cur_val} and {} with primes up to {}", smooths.get(cur+1), PRIMES[smooths.primes-1]);
                    smooths.add_prime();
                    cur = smooths.find_ind_le(cur_val).unwrap();
                },
                None => {
                    // advance normally if no gap was found
                    cur = min(cur+*NUM_THREADS*step_width, smooths.len()-1);
                },
            }
        }
    }
    println!("Done {}", smooths.get(cur-1));
}
