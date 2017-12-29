#[macro_use] extern crate rate_limit;
#[macro_use] extern crate lazy_static;
extern crate time;
use rate_limit::WallClockIntBucketCombinedMT;

fn main() {
    loop {
        limit!(2, 0.5, {
            println!("hello at {}", time::precise_time_s());
        });
    }
}
