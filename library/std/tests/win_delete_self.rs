#![cfg(windows)]

/// Attempting to delete a running binary should return an error on Windows.
#[test]
<<<<<<< HEAD
=======
#[cfg_attr(miri, ignore)] // `remove_file` does not work in Miri on Windows
>>>>>>> 30728aeafb88a31d3ab35f64dc75a07082413491
fn win_delete_self() {
    let path = std::env::current_exe().unwrap();
    assert!(std::fs::remove_file(path).is_err());
}
