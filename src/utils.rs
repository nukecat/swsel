#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            print!("{}", format!($($arg)*));
        }
    };
}