#[cfg(test)]
extern crate rand;
use std::cmp::{min, max};

pub struct FloatBucket {
    // timeunit is probably millisecond but could be anything you want (depending on overflows...)

    // static state
    max_tokens: u64,
    interval: u64, // non-zero timespan

    // dynamic state
    tokens: f64,
    last_fill_time: u64, // timespan since epoch
}

impl FloatBucket {
    // A new bucket that will be filled on first accept (given that it's later than epoch)
    pub fn new(max_tokens: u64, interval: u64) -> FloatBucket {
        if interval == 0 {
            panic!("Can't have 0 interval for Bucket");
        }

        FloatBucket {
           tokens: 0f64,
           last_fill_time: 0,
           max_tokens: max_tokens,
           interval: interval
        }
    }

    pub fn accept(&mut self, timestamp: u64) -> bool {
        let delta_time   = timestamp - self.last_fill_time;
        let delta_tokens = (self.max_tokens as f64) / (self.interval as f64) * (delta_time as f64);

        if self.tokens + delta_tokens >= 0.99f64 { // imprecisions in float makes this more correct than a strict 1f64
            self.tokens = self.tokens + delta_tokens;
            if self.tokens > self.max_tokens as f64 {
                self.tokens = self.max_tokens as f64;
            }
            self.tokens -= 1f64;
            self.last_fill_time = timestamp;

            true
        } else {
            false
        }
    }
}

pub struct IntBucket {
    // timeunit is probably millisecond but could be anything you want (depending on overflows...)

    // static state
    max_tokens: u64,
    interval: u64, // non-zero timespan

    // dynamic state
    token_time: u64,
    last_fill_time: u64, // timespan since epoch
}

impl IntBucket {
    // A new bucket that will be filled on first accept (given that it's later than epoch)
    pub fn new(max_tokens: u64, interval: u64) -> IntBucket {
        if interval == 0 {
            panic!("Can't have 0 interval for Bucket");
        }

        IntBucket {
           token_time: 0,
           last_fill_time: 0,
           max_tokens: max_tokens,
           interval: interval,
        }
    }

    pub fn accept(&mut self, timestamp: u64) -> bool {
        // no going back in time!
        let timestamp = max(timestamp, self.last_fill_time);

        let delta_time       = timestamp - self.last_fill_time;
        let delta_token_time = self.max_tokens * delta_time;
        let new_token_time   = self.token_time + delta_token_time;

        if new_token_time >= self.interval {
            self.token_time = min(self.max_tokens * self.interval, new_token_time) - self.interval;
            self.last_fill_time = timestamp;
            true
        } else {
            false
        }
    }
}

pub struct IntBucketCombined {
    // timeunit is probably millisecond but could be anything you want (depending on overflows...)

    // static state
    max_tokens: u64,
    interval: u64, // non-zero timespan

    // experimental -- using this means we're less explicit, but saving space and making
    // CAS operations easier (CAS2 is not always available and/or performant)
    // NOTE: if max_tokens is high, this will overflow. Consider max_tokens * max_timestamp (not
    // really a problem for actual plausible values of max_tokens and times. Want a thousand
    // per second until year 3000? Not even close to a problem)
    combined: u64,
}

impl IntBucketCombined {
    // A new bucket that will be filled on first accept (given that it's later than epoch)
    pub fn new(max_tokens: u64, interval: u64) -> IntBucketCombined {
        if interval == 0 {
            panic!("Can't have 0 interval for Bucket");
        }

        IntBucketCombined {
           max_tokens: max_tokens,
           interval: interval,
           combined: 0,
        }
    }

    pub fn accept(&mut self, timestamp: u64) -> bool {
        let inflated_timestamp = max(self.max_tokens * timestamp, self.combined);

        let new_token_time = inflated_timestamp - self.combined;
        if new_token_time >= self.interval {
            let token_time = min(self.max_tokens * self.interval, new_token_time) - self.interval;
            self.combined = inflated_timestamp - token_time;
            true
        } else {
            false
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use rand::isaac::IsaacRng;

    type Bucket = IntBucketCombined;

    // TODO: test going back in time

    #[test]
    fn accepts_first() {
        let mut bucket = Bucket::new(1, 10);
        assert!(bucket.accept(10000));
    }

    #[test]
    fn fail_multiple() {
        let mut bucket = Bucket::new(2, 10);
        assert!(bucket.accept(10000));
        assert!(bucket.accept(10000));
        assert!(!bucket.accept(10000));
        assert!(!bucket.accept(10000));
    }

    #[test]
    fn pass_after_time() {
        let mut bucket = Bucket::new(1, 10);
        assert!(bucket.accept(10000));
        assert!(!bucket.accept(10000));
        assert!(bucket.accept(10010));
    }

    #[test]
    fn one_per_timeunit() {
        let mut bucket = Bucket::new(3, 10);
        assert!( bucket.accept(10000)); // 20
        assert!( bucket.accept(10001)); // 13
        assert!( bucket.accept(10002)); //  6
        assert!(!bucket.accept(10003)); //  9
        assert!( bucket.accept(10004)); // 12
        assert!(!bucket.accept(10005)); //  5
        assert!(!bucket.accept(10006)); //  8
        assert!( bucket.accept(10007)); // 11
        assert!(!bucket.accept(10008)); //  4
        assert!(!bucket.accept(10009)); //  7
        assert!( bucket.accept(10010)); // 10
        assert!(!bucket.accept(10011)); //  3
        assert!(!bucket.accept(10012)); //  6
        assert!(!bucket.accept(10013)); //  9
        assert!( bucket.accept(10014)); // 12
        assert!(!bucket.accept(10015)); //  5
        assert!(!bucket.accept(10016)); //  8
        assert!( bucket.accept(10017)); // 11
        assert!(!bucket.accept(10018)); //  4
        assert!(!bucket.accept(10019)); //  7
        assert!( bucket.accept(10020)); // 10
        assert!(!bucket.accept(10021)); //  3
        assert!(!bucket.accept(10022)); //  6
        assert!(!bucket.accept(10023)); //  9
        assert!( bucket.accept(10024)); // 12
    }

    #[test]
    fn compare_implementations() {
        let mut rng = IsaacRng::new_unseeded();
        for tokens in 0..9 {
            for interval in (tokens+1)..9 {
                let mut bucket_a = IntBucket::new(tokens, interval);
                let mut bucket_b = FloatBucket::new(tokens, interval);
                //let step_sizes = 0..100;
                //let mut rng = rand::thread_rng();
                let mut timestamp = 0u64;
                println!("{} {}", tokens, interval);
                for _ in 0..10000 {
                    let step_size = (rng.next_u32() % 10) as u64;//step_sizes.ind_sample(&mut rng);
                    timestamp += step_size;
                    let a = bucket_a.accept(timestamp);
                    let b = bucket_b.accept(timestamp);
                    // println!("{} {} {}", timestamp, a, b);
                    assert!(a == b);
                }
            }
        }
    }
}
