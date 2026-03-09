pub mod bench;
pub mod cache;
pub mod config;
pub mod dense;
pub mod eval;
pub mod extract;
pub mod hybrid;
pub mod search;
pub mod segment;
pub mod system;
pub mod vector;

#[macro_export]
macro_rules! trace {
    ($level:expr, $current:expr, $($arg:tt)*) => {
        if $current >= $level {
            eprintln!($($arg)*);
        }
    };
}
