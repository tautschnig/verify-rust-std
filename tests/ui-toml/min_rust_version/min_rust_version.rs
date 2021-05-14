#![allow(clippy::redundant_clone)]
#![warn(clippy::manual_non_exhaustive)]

use std::ops::Deref;

mod enums {
    enum E {
        A,
        B,
        #[doc(hidden)]
        _C,
    }

    // user forgot to remove the marker
    #[non_exhaustive]
    enum Ep {
        A,
        B,
        #[doc(hidden)]
        _C,
    }
}

fn option_as_ref_deref() {
    let mut opt = Some(String::from("123"));

    let _ = opt.as_ref().map(String::as_str);
    let _ = opt.as_ref().map(|x| x.as_str());
    let _ = opt.as_mut().map(String::as_mut_str);
    let _ = opt.as_mut().map(|x| x.as_mut_str());
}

fn match_like_matches() {
    let _y = matches!(Some(5), Some(0));
}

fn match_same_arms() {
    match (1, 2, 3) {
        (1, .., 3) => 42,
        (.., 3) => 42, //~ ERROR match arms have same body
        _ => 0,
    };
}

fn match_same_arms2() {
    let _ = match Some(42) {
        Some(_) => 24,
        None => 24, //~ ERROR match arms have same body
    };
}

fn manual_strip_msrv() {
    let s = "hello, world!";
    if s.starts_with("hello, ") {
        assert_eq!(s["hello, ".len()..].to_uppercase(), "WORLD!");
    }
}

fn main() {
    option_as_ref_deref();
    match_like_matches();
    match_same_arms();
    match_same_arms2();
    manual_strip_msrv();
}
