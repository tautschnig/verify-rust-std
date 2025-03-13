//@aux-build: proc_macros.rs

#![allow(
    dead_code,
    unused_variables,
    overflowing_literals,
    clippy::excessive_precision,
    clippy::inconsistent_digit_grouping,
    clippy::unusual_byte_groupings
)]

extern crate proc_macros;
use proc_macros::with_span;

fn main() {
    let fail14 = 2_32;
    //~^ mistyped_literal_suffixes
    let fail15 = 4_64;
    //~^ mistyped_literal_suffixes
    let fail16 = 7_8; //
    //
    //~^^ mistyped_literal_suffixes
    let fail17 = 23_16; //
    //
    //~^^ mistyped_literal_suffixes
    let ok18 = 23_128;

    let fail20 = 2__8; //
    //
    //~^^ mistyped_literal_suffixes
    let fail21 = 4___16; //
    //
    //~^^ mistyped_literal_suffixes

    let ok24 = 12.34_64;
    let fail25 = 1E2_32;
    //~^ mistyped_literal_suffixes
    let fail26 = 43E7_64;
    //~^ mistyped_literal_suffixes
    let fail27 = 243E17_32;
    //~^ mistyped_literal_suffixes
    let fail28 = 241251235E723_64;
    //~^ mistyped_literal_suffixes
    let ok29 = 42279.911_32;

    // testing that the suggestion actually fits in its type
    let fail30 = 127_8; // should be i8
    //
    //~^^ mistyped_literal_suffixes
    let fail31 = 240_8; // should be u8
    //
    //~^^ mistyped_literal_suffixes
    let ok32 = 360_8; // doesn't fit in either, should be ignored
    let fail33 = 0x1234_16;
    //~^ mistyped_literal_suffixes
    let fail34 = 0xABCD_16;
    //~^ mistyped_literal_suffixes
    let ok35 = 0x12345_16;
    let fail36 = 0xFFFF_FFFF_FFFF_FFFF_64; // u64
    //
    //~^^ mistyped_literal_suffixes

    // issue #6129
    let ok37 = 123_32.123;
    let ok38 = 124_64.0;

    let _ = 1.12345E1_32;
    //~^ mistyped_literal_suffixes

    let _ = with_span!(1 2_u32);
}
