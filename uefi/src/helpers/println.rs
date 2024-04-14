use crate::helpers::system_table;
use core::fmt::Write;

/// INTERNAL API! Helper for print macros.
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    system_table()
        .stdout()
        .write_fmt(args)
        .expect("Failed to write to stdout");
}

/// Prints to the standard output.
///
/// # Panics
/// Will panic if `SYSTEM_TABLE` is `None` (Before [`uefi::helpers::init()`] and
/// after [`uefi::prelude::SystemTable::exit_boot_services()`]).
///
/// # Examples
/// ```
/// print!("");
/// print!("Hello World\n");
/// print!("Hello {}", "World");
/// ```
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::helpers::_print(core::format_args!($($arg)*)));
}

/// Prints to the standard output, with a newline.
///
/// # Panics
/// Will panic if `SYSTEM_TABLE` is `None` (Before [`uefi::helpers::init()`] and
/// after [`uefi::prelude::SystemTable::exit_boot_services()`]).
///
/// # Examples
/// ```
/// println!();
/// println!("Hello World");
/// println!("Hello {}", "World");
/// ```
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::helpers::_print(core::format_args!("{}{}", core::format_args!($($arg)*), "\n")));
}
