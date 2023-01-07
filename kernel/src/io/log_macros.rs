#[macro_export]
macro_rules! fatal {
    ($($t:tt)*) => { $crate::println!("{}[FATAL] {}{}", crate::io::colours::RED_BG, format_args!($($t)*), crate::io::colours::BLACK_BG); };
}

#[macro_export]
macro_rules! error {
    ($($t:tt)*) => { $crate::println!("{}[ERROR]{} {}", crate::io::colours::RED_FG, crate::io::colours::WHITE_FG, format_args!($($t)*)); };
}

#[macro_export]
macro_rules! warn {
    ($($t:tt)*) => { $crate::println!("{}[WARN]{} {}", crate::io::colours::YELLOW_FG, crate::io::colours::WHITE_FG, format_args!($($t)*)); };
}

#[macro_export]
macro_rules! info {
    ($($t:tt)*) => { $crate::println!("{}[INFO]{} {}", crate::io::colours::GREEN_FG, crate::io::colours::WHITE_FG, format_args!($($t)*)); };
}

#[macro_export]
macro_rules! trace {
    ($($t:tt)*) => { $crate::println!("{}[TRACE]{} {}", crate::io::colours::CYAN_FG, crate::io::colours::WHITE_FG, format_args!($($t)*)); };
}
