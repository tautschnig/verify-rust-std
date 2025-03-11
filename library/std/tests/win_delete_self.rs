#![cfg(windows)]

/// Attempting to delete a running binary should return an error on Windows.
#[test]
<<<<<<< HEAD
=======
#[cfg_attr(miri, ignore)] // `remove_file` does not work in Miri on Windows
>>>>>>> 4fc84ab1659ac7975991ec71d645ebe7c240376b
fn win_delete_self() {
    let path = std::env::current_exe().unwrap();
    assert!(std::fs::remove_file(path).is_err());
}
