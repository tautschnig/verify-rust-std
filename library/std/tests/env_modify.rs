// These tests are in a separate integration test as they modify the environment,
// and would otherwise cause some other tests to fail.

use std::env::*;
use std::ffi::{OsStr, OsString};

<<<<<<< HEAD
use rand::distributions::{Alphanumeric, DistString};
=======
use rand::distr::{Alphanumeric, SampleString};
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491

mod common;
use std::thread;

use common::test_rng;

#[track_caller]
fn make_rand_name() -> OsString {
    let n = format!("TEST{}", Alphanumeric.sample_string(&mut test_rng(), 10));
    let n = OsString::from(n);
    assert!(var_os(&n).is_none());
    n
}

fn eq(a: Option<OsString>, b: Option<&str>) {
    assert_eq!(a.as_ref().map(|s| &**s), b.map(OsStr::new).map(|s| &*s));
}

#[test]
fn test_set_var() {
    let n = make_rand_name();
<<<<<<< HEAD
    set_var(&n, "VALUE");
=======
    unsafe {
        set_var(&n, "VALUE");
    }
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
    eq(var_os(&n), Some("VALUE"));
}

#[test]
fn test_remove_var() {
    let n = make_rand_name();
<<<<<<< HEAD
    set_var(&n, "VALUE");
    remove_var(&n);
=======
    unsafe {
        set_var(&n, "VALUE");
        remove_var(&n);
    }
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
    eq(var_os(&n), None);
}

#[test]
fn test_set_var_overwrite() {
    let n = make_rand_name();
<<<<<<< HEAD
    set_var(&n, "1");
    set_var(&n, "2");
    eq(var_os(&n), Some("2"));
    set_var(&n, "");
    eq(var_os(&n), Some(""));
=======
    unsafe {
        set_var(&n, "1");
        set_var(&n, "2");
        eq(var_os(&n), Some("2"));
        set_var(&n, "");
        eq(var_os(&n), Some(""));
    }
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
}

#[test]
#[cfg_attr(target_os = "emscripten", ignore)]
fn test_var_big() {
    let mut s = "".to_string();
    let mut i = 0;
    while i < 100 {
        s.push_str("aaaaaaaaaa");
        i += 1;
    }
    let n = make_rand_name();
<<<<<<< HEAD
    set_var(&n, &s);
=======
    unsafe {
        set_var(&n, &s);
    }
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
    eq(var_os(&n), Some(&s));
}

#[test]
#[cfg_attr(target_os = "emscripten", ignore)]
fn test_env_set_get_huge() {
    let n = make_rand_name();
    let s = "x".repeat(10000);
<<<<<<< HEAD
    set_var(&n, &s);
    eq(var_os(&n), Some(&s));
    remove_var(&n);
    eq(var_os(&n), None);
=======
    unsafe {
        set_var(&n, &s);
        eq(var_os(&n), Some(&s));
        remove_var(&n);
        eq(var_os(&n), None);
    }
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
}

#[test]
fn test_env_set_var() {
    let n = make_rand_name();

    let mut e = vars_os();
<<<<<<< HEAD
    set_var(&n, "VALUE");
=======
    unsafe {
        set_var(&n, "VALUE");
    }
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
    assert!(!e.any(|(k, v)| { &*k == &*n && &*v == "VALUE" }));

    assert!(vars_os().any(|(k, v)| { &*k == &*n && &*v == "VALUE" }));
}

#[test]
#[cfg_attr(not(any(unix, windows)), ignore, allow(unused))]
#[allow(deprecated)]
fn env_home_dir() {
    use std::path::PathBuf;

    fn var_to_os_string(var: Result<String, VarError>) -> Option<OsString> {
        match var {
            Ok(var) => Some(OsString::from(var)),
            Err(VarError::NotUnicode(var)) => Some(var),
            _ => None,
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(unix)] {
            let oldhome = var_to_os_string(var("HOME"));

<<<<<<< HEAD
            set_var("HOME", "/home/MountainView");
            assert_eq!(home_dir(), Some(PathBuf::from("/home/MountainView")));

            remove_var("HOME");
=======
            unsafe {
                set_var("HOME", "/home/MountainView");
                assert_eq!(home_dir(), Some(PathBuf::from("/home/MountainView")));

                remove_var("HOME");
            }
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
            if cfg!(target_os = "android") {
                assert!(home_dir().is_none());
            } else {
                // When HOME is not set, some platforms return `None`,
                // but others return `Some` with a default.
                // Just check that it is not "/home/MountainView".
                assert_ne!(home_dir(), Some(PathBuf::from("/home/MountainView")));
            }

<<<<<<< HEAD
            if let Some(oldhome) = oldhome { set_var("HOME", oldhome); }
=======
            if let Some(oldhome) = oldhome { unsafe { set_var("HOME", oldhome); } }
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
        } else if #[cfg(windows)] {
            let oldhome = var_to_os_string(var("HOME"));
            let olduserprofile = var_to_os_string(var("USERPROFILE"));

<<<<<<< HEAD
            remove_var("HOME");
            remove_var("USERPROFILE");

            assert!(home_dir().is_some());

            set_var("HOME", "/home/PaloAlto");
            assert_ne!(home_dir(), Some(PathBuf::from("/home/PaloAlto")), "HOME must not be used");

            set_var("USERPROFILE", "/home/MountainView");
            assert_eq!(home_dir(), Some(PathBuf::from("/home/MountainView")));

            remove_var("HOME");

            assert_eq!(home_dir(), Some(PathBuf::from("/home/MountainView")));

            set_var("USERPROFILE", "");
            assert_ne!(home_dir(), Some(PathBuf::from("")), "Empty USERPROFILE must be ignored");

            remove_var("USERPROFILE");

            if let Some(oldhome) = oldhome { set_var("HOME", oldhome); }
            if let Some(olduserprofile) = olduserprofile { set_var("USERPROFILE", olduserprofile); }
=======
            unsafe {
                remove_var("HOME");
                remove_var("USERPROFILE");

                assert!(home_dir().is_some());

                set_var("HOME", "/home/PaloAlto");
                assert_ne!(home_dir(), Some(PathBuf::from("/home/PaloAlto")), "HOME must not be used");

                set_var("USERPROFILE", "/home/MountainView");
                assert_eq!(home_dir(), Some(PathBuf::from("/home/MountainView")));

                remove_var("HOME");

                assert_eq!(home_dir(), Some(PathBuf::from("/home/MountainView")));

                set_var("USERPROFILE", "");
                assert_ne!(home_dir(), Some(PathBuf::from("")), "Empty USERPROFILE must be ignored");

                remove_var("USERPROFILE");

                if let Some(oldhome) = oldhome { set_var("HOME", oldhome); }
                if let Some(olduserprofile) = olduserprofile { set_var("USERPROFILE", olduserprofile); }
            }
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
        }
    }
}

#[test] // miri shouldn't detect any data race in this fn
#[cfg_attr(any(not(miri), target_os = "emscripten"), ignore)]
fn test_env_get_set_multithreaded() {
    let getter = thread::spawn(|| {
        for _ in 0..100 {
            let _ = var_os("foo");
        }
    });

    let setter = thread::spawn(|| {
        for _ in 0..100 {
<<<<<<< HEAD
            set_var("foo", "bar");
=======
            unsafe {
                set_var("foo", "bar");
            }
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
        }
    });

    let _ = getter.join();
    let _ = setter.join();
}
