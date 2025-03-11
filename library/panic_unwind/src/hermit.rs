//! Unwinding for *hermit* target.
//!
//! Right now we don't support this, so this is just stubs.

use alloc::boxed::Box;
use core::any::Any;

pub(crate) unsafe fn cleanup(_ptr: *mut u8) -> Box<dyn Any + Send> {
<<<<<<< HEAD
    extern "C" {
        fn __rust_abort() -> !;
=======
    unsafe extern "C" {
        fn __rust_abort() -> !;
    }
    unsafe {
        __rust_abort();
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
    }
}

pub(crate) unsafe fn panic(_data: Box<dyn Any + Send>) -> u32 {
<<<<<<< HEAD
    extern "C" {
        fn __rust_abort() -> !;
=======
    unsafe extern "C" {
        fn __rust_abort() -> !;
    }
    unsafe {
        __rust_abort();
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
    }
}
