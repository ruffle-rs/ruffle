
// This entire file os only needed for the Oboe host of the Cpal audio backend,
// see: https://github.com/katyo/oboe-rs/issues/28#issuecomment-1001103335

fn add_lib(_name: impl AsRef<str>, _static: bool) {
    #[cfg(not(feature = "test"))]
    println!(
        "cargo:rustc-link-lib={}{}",
        if _static { "static=" } else { "" },
        _name.as_ref()
    );
}

fn main() {
    add_lib("c++_shared", false);
}