//! Logging macros

#[macro_export]
/// Fatal error. Prints the entire message in white, with a red background.
macro_rules! fatal {
    ($($t:tt)*) => { $crate::println!("{}[FATAL] {}{}", crate::io::colours::RED_BG, format_args!($($t)*), crate::io::colours::BLACK_BG); };
}

#[macro_export]
/// Error. Prefix printed in red.
macro_rules! error {
    ($($t:tt)*) => { $crate::println!("{}[ERROR]{} {}", crate::io::colours::RED_FG, crate::io::colours::WHITE_FG, format_args!($($t)*)); };
}

#[macro_export]
/// Warning. Prefix printed in yellow.
macro_rules! warn {
    ($($t:tt)*) => { $crate::println!("{}[WARN]{} {}", crate::io::colours::YELLOW_FG, crate::io::colours::WHITE_FG, format_args!($($t)*)); };
}

#[macro_export]
/// Info. Prefix printed in green.
macro_rules! info {
    ($($t:tt)*) => { $crate::println!("{}[INFO]{} {}", crate::io::colours::GREEN_FG, crate::io::colours::WHITE_FG, format_args!($($t)*)); };
}

#[macro_export]
/// Trace. Prefix printed in cyan.
macro_rules! trace {
    ($($t:tt)*) => { $crate::println!("{}[TRACE]{} {}", crate::io::colours::CYAN_FG, crate::io::colours::WHITE_FG, format_args!($($t)*)); };
}
