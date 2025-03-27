//! Test that linking a no_std application still outputs the
//! `native-static-libs: ` note, even though it is empty.

//@ compile-flags: -Cpanic=abort --print=native-static-libs
//@ build-pass
//@ error-pattern: note: native-static-libs:
//@ dont-check-compiler-stderr (libcore links `/defaultlib:msvcrt` or `/defaultlib:libcmt` on MSVC)
//@ ignore-pass (the note is emitted later in the compilation pipeline, needs build)

#![crate_type = "staticlib"]
#![no_std]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
