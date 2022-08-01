//! Functions to determine the host platform.
//!
//! Use the functions where possible instead of `#[cfg(...)]` so that
//! code for all platforms gets checked at compile time.

use std::env::consts;

pub fn is_linux() -> bool {
    consts::OS == "linux"
}

pub fn is_unix() -> bool {
    consts::FAMILY == "unix"
}

pub fn is_windows() -> bool {
    consts::FAMILY == "windows"
}
