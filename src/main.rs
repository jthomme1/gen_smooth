use crate::smooths::Smooths;
use std::vec::Vec;
use once_cell::sync::Lazy;
use primal;
use std::env;

pub mod composite;
pub mod smooths;

// this should suffice for now
static PRIME_BOUND: usize = 1<<12;
static PRIMES: Lazy<Vec<u64>> = Lazy::new(||
                                           primal::Sieve::new(PRIME_BOUND)
                                           .primes_from(0)
                                           .collect::<Vec<usize>>()
                                           .into_iter()
                                           .map(|x| u64::try_from(x).unwrap())
                                           .collect());

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Provide exactly one argument (log upper bound).");
        return;
    }
    let log_n = usize::from_str_radix(&args[1], 10).unwrap();
    assert!(log_n % 2 == 0 && log_n > 2);
    // index to the current smooth number we consider
    // accessing PRIMES triggers its generataion
    println!("{} primes generated.", PRIMES.len());
    let mut smooths = Smooths::new(log_n);
    while smooths.full_filled != log_n/2 {
        smooths.add_prime();
    }
}
