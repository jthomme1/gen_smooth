use crate::smooths::Smooths;
use std::vec::Vec;
use once_cell::sync::Lazy;
use integer_sqrt::IntegerSquareRoot;
use primal;
use std::thread;
use std::cmp::min;
use std::env;

pub mod composite;
pub mod smooths;

// this should suffice for now
static PRIME_POWER_BOUND: usize = 2<<10;
static PRIMES: Lazy<Vec<usize>> = Lazy::new(|| primal::Sieve::new(PRIME_POWER_BOUND).primes_from(0).collect());
static PRIME_POWERS: Lazy<Vec<usize>> = Lazy::new(|| {
                                                  let mut pp = vec![];
                                                  for p in PRIMES.iter() {
                                                      let mut cur: usize = *p;
                                                      while cur <= PRIME_POWER_BOUND {
                                                          pp.push(cur);
                                                          cur *= p;
                                                      }
                                                  }
                                                  pp.sort();
                                                  pp
});
static NUM_THREADS: Lazy<usize> = Lazy::new(|| thread::available_parallelism().unwrap().get());

fn main() {
    // accessing PRIMES triggers its generataion
    println!("Now generating primes.");
    println!("{} primes generated.", PRIMES.len());
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Provide exactly one argument (upper bound).");
        return;
    }
    let n = u64::from_str_radix(&args[1], 10).unwrap();
    // index to the current smooth number we consider
    let mut smooths = Smooths::new(n);
    let mut cur: usize = smooths.find_ind_le(2).unwrap();
    // the interval covered by a smooth number
    let right = |x: u64| {
        if u64::MAX - (4u64*x).integer_sqrt() - 1u64 < x {
            return u64::MAX;
        }
        x + (4u64*x).integer_sqrt() + 1u64
    };
    let left = |x: u64| {x - (4u64*x).integer_sqrt() + 1u64};
    // fn to get the index of the biggest prime below the bound for val and c
    println!("Detected {}-parallelism.", *NUM_THREADS);
    // iterate over current range of smooth numbers
    // we go only until the second to last element as there is no gap to consider for the last
    // element
    while right(smooths.get(cur)) < n {
        if cur == smooths.len() - 1 {
            // we have a gap because we are out of powersmooth numbers
            let cur_val = smooths.get(cur);
            println!("Gap at {cur_val} with prime powers up to {}", PRIME_POWERS[smooths.pp_ind-1]);
            smooths.add_prime_power(cur_val);
            cur = smooths.find_ind_le(cur_val).unwrap();
            assert!(smooths.get(cur) == cur_val)
        } else {
            // since it is really rare that there is no smooth number in the interval of
            // interest, we parallelize the search
            let step_width: usize = 1 << 20;
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
                    let next_val = smooths.get(cur + 1);
                    println!("Gap between {cur_val} and {next_val} with prime powers up to {}", PRIME_POWERS[smooths.pp_ind-1]);
                    smooths.add_prime_power(cur_val);
                    cur = smooths.find_ind_le(cur_val).unwrap();
                    assert!(smooths.get(cur) == cur_val)
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
