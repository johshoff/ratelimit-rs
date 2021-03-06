#![feature(test)]
extern crate test;
extern crate rate_limit;

use rate_limit::*;
use test::Bencher;
use std::sync::Mutex;

#[bench]
fn bench_add_two_combined(b: &mut Bencher) {
    let mut bucket = IntBucketCombined::new(3, 10);
    let mut timestamp = 0;
    b.iter(|| {
        for _ in 0..1000 {
            timestamp += 1;
            bucket.accept(timestamp);
        }
        bucket.accept(timestamp)
    });
}

#[bench]
fn bench_add_two_int(b: &mut Bencher) {
    let mut bucket = IntBucket::new(3, 10);
    let mut timestamp = 0;
    b.iter(|| {
        for _ in 0..1000 {
            timestamp += 1;
            bucket.accept(timestamp);
        }
        bucket.accept(timestamp)
    });
}

#[bench]
fn bench_add_two_float(b: &mut Bencher) {
    let mut bucket = FloatBucket::new(3, 10);
    let mut timestamp = 0;
    b.iter(|| {
        for _ in 0..1000 {
            timestamp += 1;
            bucket.accept(timestamp);
        }
        bucket.accept(timestamp)
    });
}

#[bench]
fn bench_add_two_mt(b: &mut Bencher) {
    let bucket = IntBucketCombinedMT::new(3, 10);
    let mut timestamp = 0;
    b.iter(|| {
        for _ in 0..1000 {
            timestamp += 1;
            bucket.accept(timestamp);
        }
        bucket.accept(timestamp)
    });
}

#[bench]
fn bench_add_two_mutex(b: &mut Bencher) {
    let bucket = Mutex::new(IntBucket::new(3, 10));
    let mut timestamp = 0;
    b.iter(|| {
        for _ in 0..1000 {
            timestamp += 1;
            bucket.lock().unwrap().accept(timestamp);
        }
        bucket.lock().unwrap().accept(timestamp)
    });
}
