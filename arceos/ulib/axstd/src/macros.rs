//! Standard library macros

/// Prints to the standard output.
///
/// Equivalent to the [`println!`] macro except that a newline is not printed at
/// the end of the message.
///
/// [`println!`]: crate::println
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        unsafe { $crate::io::__print_impl(format_args!($($arg)*)); }
    }
}

/// Prints to the standard output, with a newline.
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {
        unsafe {
            $crate::io::__print_impl(format_args!($($arg)*));
            $crate::io::__print_impl(format_args!("\n"));
        }
    }
}
