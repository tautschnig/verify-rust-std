<<<<<<<< HEAD:library/std/tests/sync/mpmc.rs
use std::sync::mpmc::*;
use std::time::{Duration, Instant};
use std::{env, thread};

pub fn stress_factor() -> usize {
    match env::var("RUST_TEST_STRESS") {
        Ok(val) => val.parse().unwrap(),
        Err(..) => 1,
    }
}

========
// Ensure that thread_local init with `const { 0 }` still has unique address at run-time
>>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491:library/std/src/sync/mpmc/tests.rs
#[test]
fn waker_current_thread_id() {
    let first = super::waker::current_thread_id();
    let t = crate::thread::spawn(move || {
        let second = super::waker::current_thread_id();
        assert_ne!(first, second);
        assert_eq!(second, super::waker::current_thread_id());
    });

    assert_eq!(first, super::waker::current_thread_id());
    t.join().unwrap();
    assert_eq!(first, super::waker::current_thread_id());
}
