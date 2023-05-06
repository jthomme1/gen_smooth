use std::vec::Vec;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::cmp::{max, min};
use crate::composite::Composite;
use bitvec::prelude::*;
use integer_sqrt::IntegerSquareRoot;
use once_cell::sync::Lazy;
use super::PRIMES;

static NUM_THREADS: Lazy<usize> = Lazy::new(|| thread::available_parallelism().unwrap().get());

fn counter_ind_to_bound(ind: usize) -> u64 {
    1u64<<(2*(ind+1))
}

fn bucket_ind_to_counter_ind(ind: usize) -> usize {
    usize::try_from((ind+1).ilog2()).unwrap()
}

fn counter_limit(ind: usize) -> usize {
    (1<<(ind+1))-1
}

fn val_to_bucket_ind(val: u64) -> usize {
    usize::try_from(val.integer_sqrt()).unwrap()-1
}

// this needs to be a power of 2
fn nr_bitvecs(log_bound: usize) -> usize {
    min(1<<((log_bound/2).ilog2()), *NUM_THREADS*2)
}

fn interval_width(log_bound: usize) -> usize {
    (1<<(log_bound/2))/nr_bitvecs(log_bound)
}

pub struct Smooths {
    pub log_bound: usize,
    pub primes: usize,
    // ~ for each even power of 2
    pub full_counters: Vec<usize>,
    // we check if there are no consecutive non-covered intervals, equivalently, every interval is
    // covered by its neighbors
    pub alt_counters: Vec<usize>,
    pub intervals: Arc<Vec<Mutex<BitVec>>>,
    pub full_filled: usize,
    pub alt_filled: usize,
}

impl Smooths {
    pub fn new(log_bound: usize) -> Self {
        let mut full_counters = vec![0; log_bound/2];
        let mut alt_counters = vec![0; log_bound/2];
        let mut tmp_mtxs = vec![];
        for _ in 0..nr_bitvecs(log_bound) {
            tmp_mtxs.push(Mutex::new(bitvec![0; interval_width(log_bound)]));
        }
        let intervals = Arc::new(tmp_mtxs);

        println!("NUM_THREADS: {}, nr_bitvecs: {}, interval_width: {}", *NUM_THREADS, nr_bitvecs(log_bound), interval_width(log_bound));
        let intw = interval_width(log_bound);
        // we add 2**0 to 2**log_bound to avoid some edge cases later
        for i in 0..=log_bound {
            let ind = val_to_bucket_ind(1<<i);
            let lock_ind = ind/intw;
            let int_ind = ind%intw;
            //println!("val: {}, ind: {ind}, lock_ind: {lock_ind}, int_ind: {int_ind}", 1<<i);
            (*intervals[lock_ind].lock().unwrap()).set(int_ind, true);
        }

        for i in 0..log_bound/2 {
            if i == 0 {
                alt_counters[i] = 2;
                full_counters[i] = 1;
            } else if i == 1 {
                alt_counters[i] = 6;
                full_counters[i] = 2;
            } else if i == 2 {
                alt_counters[i] = 12;
                full_counters[i] = full_counters[i-1] + 2;
            } else {
                alt_counters[i] = alt_counters[i-1] + 8;
                full_counters[i] = full_counters[i-1] + 2;
            }
        }
        Smooths {
            log_bound: log_bound,
            primes: 1,
            full_counters: full_counters,
            alt_counters: alt_counters,
            intervals: intervals,
            full_filled: 1,
            alt_filled: 2,
        }
    }

    pub fn add_prime(&mut self) {
        self.init_gen(self.primes);
        println!("Done with {}", PRIMES[self.primes]);
        for i in 0..self.full_counters.len() {
            //println!("{i}: full_cover: {}, alt_cover: {}, limit: {}", self.full_counters[i], self.alt_counters[i], counter_limit(i));
            let val = counter_ind_to_bound(i).ilog2();
            //let c = (PRIMES[self.primes] as f64).log(val as f64);
            if self.full_counters[i] == counter_limit(i) && self.full_filled <= i {
                //println!("2**{}: c <= {}", val, c);
                println!("2**{}: fully covered", val);
                self.full_filled = i+1;
            }
            if self.alt_counters[i] == 2*counter_limit(i) && self.alt_filled <= i {
                //println!("2**{}: c >= {}", val, c);
                println!("2**{}: alternatingly covered", val);
                self.alt_filled = i+1;
            }
        }
        self.primes += 1;
    }

    fn insert_smooths(smooths: &mut Vec<u64>, intervals: Arc<Vec<Mutex<BitVec>>>, log_bound: usize) -> (Vec<usize>, Vec<usize>) {
        let intw = interval_width(log_bound);
        let mut full_counters = vec![0; log_bound/2];
        let mut alt_counters = vec![0; log_bound/2];
        smooths.sort_unstable();
        // find the indices of numbers from where on we need to lock a mutex until we need to
        // lock it
        // starts at the first index that needs the lock until the first index that doesn't need it
        // anymore
        let mut switch_ind = vec![];
        for i in 0..nr_bitvecs(log_bound) {
            // we need to lock a mutex starting from the interval preceeding the first interval
            // covered by the mutex
            let first_int = if i == 0 { 0 } else { i*intw-1 };
            let last_int = if i == nr_bitvecs(log_bound)-1 { (i+1)*intw-1 } else { (i+1)*intw };
            //println!("{:?}: We need lock {i} from interval {first_int} to {last_int}", thread::current().id());
            let first_num = u64::try_from((first_int+1)*(first_int+1)).unwrap();
            let last_num = u64::try_from((last_int+2)*(last_int+2)-1).unwrap();
            //println!("{:?}: For lock {i}, this corresponds to the numbers from {first_num} to {last_num}", thread::current().id());
            // the index when we need to lock the mutex
            let first_ind = match smooths.binary_search(&first_num) {
                Ok(i) => i,
                Err(i) => i,
            };
            // the index when we need to unlock the mutex
            let last_ind = match smooths.binary_search(&last_num) {
                Ok(i) => i,
                Err(i) => i,
            };
            //println!("{:?}: For this thread and lock {i}, the corresponding smooth numbers are: {} (with ind {first_ind}) to {} (with ind {last_ind})", thread::current().id(), smooths[first_ind], smooths[last_ind]);
            switch_ind.push((first_ind, last_ind));
            //println!("{:?}: All in all, we need lock {i} from index {first_ind} to {last_ind}", thread::current().id());
        }
        let set_interval = |ind: usize, locks: &mut Vec<MutexGuard<'_, BitVec>>, cur_lock_ind: usize| {
            let lock_ind = ind/intw;
            let int_ind = ind%intw;
            // not faster unchecked
            locks[lock_ind-cur_lock_ind].set(int_ind, true);
        };

        let get_interval = |ind: usize, locks: &Vec<MutexGuard<'_, BitVec>>, cur_lock_ind: usize| -> bool {
            let lock_ind = ind/intw;
            let int_ind = ind%intw;
            // not faster unchecked
            *locks[lock_ind-cur_lock_ind].get(int_ind).unwrap()
        };

        let mut insert = |ind: usize, /* val: u64,*/locks: &mut Vec<MutexGuard<'_, BitVec>>, cur_lock_ind: usize| {
            //println!("Checking for {val} with ind {ind}, len_locks: {}", locks.len());
            if !get_interval(ind, &locks, cur_lock_ind) {
                //println!("Setting {ind} with val {val}");
                set_interval(ind, locks, cur_lock_ind);

                // we can do this without having to worry about the bounds as the edge cases
                // (intervals of the next counter covering an interval in a counter before) have
                // been taken care of in the initialization by the insertion of the powers of 2
                let prev = get_interval(ind-1, &locks, cur_lock_ind);
                let after = get_interval(ind+1, &locks, cur_lock_ind);

                let mut add = 2;
                if prev {
                    add -= 1;
                } else {
                    add += 1;
                }
                if after {
                    add -= 1;
                } else {
                    add += 1;
                }

                let range = bucket_ind_to_counter_ind(ind)..log_bound/2;

                // we are never in the first interval of a counter
                // neither in the last overall
                // also, the first interval of the next counter is already covered by a power of 2
                for i in range {
                    full_counters[i] += 1;
                    alt_counters[i] += add;
                }
            }
        };

        let mut locks = vec![intervals[0].lock().unwrap()];
        for i in 0..nr_bitvecs(log_bound) {
            // invariant: here, only intervals[i] is locked
            // start with the numbers where only intervals[i] is needed
            let from_s = if i == 0 {
                switch_ind[i].0
            } else {
                // whatever comes later: the start of the current lock or one after the end of the
                // previous one
                max(switch_ind[i].0, switch_ind[i-1].1)
            };
            let to_s = if i == nr_bitvecs(log_bound)-1 {
                switch_ind[i].1
            } else {
                // whatever comes first: the end of the current lock + 1 or the start of the
                // next one
                min(switch_ind[i].1, switch_ind[i+1].0)
            };
            //println!("Lock {i} from {from_s} to {to_s}");
            for s in from_s..to_s {
                insert(val_to_bucket_ind(smooths[s]), &mut locks, i);
            }
            // consider the number where intervals[i] and intervals[i+1] is needed
            if i != nr_bitvecs(log_bound)-1 {
                locks.push(intervals[i+1].lock().unwrap());
                let from_m = switch_ind[i+1].0;
                let to_m = switch_ind[i].1;
                //println!("Lock {i} and {} from {from_m} to {to_m}", i+1);
                for s in from_m..to_m {
                    insert(val_to_bucket_ind(smooths[s]), &mut locks, i);
                }
            }
            drop(locks.remove(0));
        }
        smooths.clear();
        (full_counters, alt_counters)
        /*
        let mut locks = vec![intervals[0].lock().unwrap()];
        let mut cur_lock_ind = 0;
        for i in 0..smooths.len() {
            let val = smooths[i];
            let ind = val_to_bucket_ind(val);
            println!("{:?}: BEGIN i: {i}, val: {val}, ind: {ind}, lock_len: {}", thread::current().id(), locks.len());

            if cur_lock_ind != nr_bitvecs(log_bound)-1 {
                if i == switch_ind[cur_lock_ind+1].0 {
                    locks.push(intervals[cur_lock_ind+1].lock().unwrap());
                     println!("{:?}: JUST LOCKED i: {i}, val: {val}, ind: {ind}, lock_ind: {}", thread::current().id(), cur_lock_ind+1);
                    // we count up if there was no other lock locked
                    if locks.len() == 1 {
                        cur_lock_ind += 1;
                    }
                }
            }
            /* INSERTION START */
            // we already set the count for the first and last bucket as they will be filled by the
            // powers of 2 either way. This way we don't have to check the border cases every time
            // -> check if last bucket really will be filled

            let set_interval = |ind: usize, locks: &mut Vec<MutexGuard<'_, BitVec>>| {
                let lock_ind = ind/intw;
                let int_ind = ind%intw;
                // TODO: make unchecked again
                locks[lock_ind-cur_lock_ind].set(int_ind, true);
            };

            let get_interval = |ind: usize, locks: &Vec<MutexGuard<'_, BitVec>>| -> bool {
                let lock_ind = ind/intw;
                let int_ind = ind%intw;
                // TODO: make unchecked again
                *locks[lock_ind-cur_lock_ind].get(int_ind).unwrap()
            };

            if !get_interval(ind, &locks) {
                //println!("Setting {ind} with val {val}");
                set_interval(ind, &mut locks);

                // we can do this without having to worry about the bounds as the edge cases have
                // been taken care of in the initialization
                let prev = get_interval(ind-1, &locks);
                let after = get_interval(ind+1, &locks);

                let mut add = 2;
                if prev {
                    add -= 1;
                } else {
                    add += 1;
                }
                if after {
                    add -= 1;
                } else {
                    add += 1;
                }

                let range = bucket_ind_to_counter_ind(ind)..log_bound/2;

                // we are never in the first interval of a counter
                // neither in the last overall
                // also, the first interval of the next counter is already covered by a power of 2
                for i in range {
                    full_counters[i] += 1;
                    alt_counters[i] += add;
                }
            }
            /* INSERTION END */
            // drop the lock after dealing with the smooth number
            if i == switch_ind[cur_lock_ind].1 {
                drop(locks.remove(0));
                println!("{:?}: UNLOCKED i: {i}, val: {val}, ind: {ind}, lock_ind: {}", thread::current().id(), cur_lock_ind);
                // we count up if there is another lock already locked
                if locks.len() != 0 {
                    cur_lock_ind += 1;
                }
            }
            println!("{:?}: AFTER DEL i: {i}, val: {val}, ind: {ind}, lock_len: {}", thread::current().id(), locks.len());
        }
        smooths.clear();
        (full_counters, alt_counters)
        */
    }

    fn init_gen(&mut self, ind: usize) {
        let log_bound = self.log_bound;
        let intervals = self.intervals.clone();
        let generate_with_fixed = |e_val: u32| {
            let mut c = Composite::new(ind, e_val);

            let mut full_counters = vec![0; log_bound/2];
            let mut alt_counters = vec![0; log_bound/2];

            let mut smooths: Vec<u64> = vec![];

            let mut accumulate = |smooths: &mut Vec<u64>| {
                //println!("Inserting {:?}", smooths);
                let (new_full, new_alt) = Self::insert_smooths(smooths, intervals.clone(), log_bound);
                for i in 0..log_bound/2 {
                    full_counters[i] += new_full[i];
                    alt_counters[i] += new_alt[i];
                }
            };
            let mut cap = 0;
            while c.es[ind] == e_val {
                smooths.push(c.value);
                cap += 1;
                if cap == 1<<20 {
                    accumulate(&mut smooths);
                    cap = 0;
                }
                c.inc_vec_with_bound(1u64<<log_bound);
            }
            accumulate(&mut smooths);
            (full_counters, alt_counters)
        };
        // for each possible exponent we start a thread
        let (full_counters, alt_counters) = thread::scope(|s| {
            let mut handles = vec![];
            let p = PRIMES[ind];
            let mut q = p;
            let mut i = 1;
            while q <= 1<<log_bound {
                let h = s.spawn(move || generate_with_fixed(i));
                handles.push(h);
                i += 1;
                q *= p;
            }
            handles.into_iter()
                .map(|h| h.join().unwrap())
                .fold((vec![0; log_bound/2], vec![0; log_bound/2]),
                    |(mut full_acc, mut alt_acc), (full, alt)| {
                        for i in 0..log_bound/2 {
                            full_acc[i] += full[i];
                            alt_acc[i] += alt[i];
                        }
                        (full_acc, alt_acc)
                    })
        });
        for i in 0..log_bound/2 {
            self.full_counters[i] += full_counters[i];
            self.alt_counters[i] += alt_counters[i];
        }
    }
}

