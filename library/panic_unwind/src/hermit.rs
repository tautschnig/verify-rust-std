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
>>>>>>> 4fc84ab1659ac7975991ec71d645ebe7c240376b
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
>>>>>>> 4fc84ab1659ac7975991ec71d645ebe7c240376b
    }
}
