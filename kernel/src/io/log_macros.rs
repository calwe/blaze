//! Logging macros

use core::sync::atomic::AtomicUsize;

/// The maximum log level. This is set by the kernel command line. (or the `set_log_level` function)
pub static MAX_LOG_LEVEL: AtomicUsize = AtomicUsize::new(4);

/// The log levels.
pub enum LogLevel {
    /// Fatal error. Prints the entire message in white, with a red background.
    Fatal = 0,
    /// Error. Prefix printed in red.
    Error = 1,
    /// Warning. Prefix printed in yellow.
    Warn = 2,
    /// Info. Prefix printed in green.
    Info = 3,
    /// Trace. Prefix printed in blue.
    Trace = 4,
}

/// Set the maximum log level.
pub fn set_log_level(level: LogLevel) {
    MAX_LOG_LEVEL.store(level as usize, core::sync::atomic::Ordering::Relaxed);
}

#[macro_export]
/// Fatal error. Prints the entire message in white, with a red background.
macro_rules! fatal {
    ($($t:tt)*) => {
        if $crate::io::log_macros::MAX_LOG_LEVEL.load(core::sync::atomic::Ordering::Relaxed) >= $crate::io::log_macros::LogLevel::Fatal as usize {
            $crate::println!("{}[FATAL] {}{}", crate::io::colours::RED_BG, format_args!($($t)*), crate::io::colours::BLACK_BG);
        }
    };
}

#[macro_export]
/// Error. Prefix printed in red.
macro_rules! error {
    ($($t:tt)*) => {
        if $crate::io::log_macros::MAX_LOG_LEVEL.load(core::sync::atomic::Ordering::Relaxed) >= $crate::io::log_macros::LogLevel::Error as usize {
            $crate::println!("{}[ERROR]{} {}", crate::io::colours::RED_FG, crate::io::colours::WHITE_FG, format_args!($($t)*));
        }
    };
}

#[macro_export]
/// Warning. Prefix printed in yellow.
macro_rules! warn {
    ($($t:tt)*) => {
        if $crate::io::log_macros::MAX_LOG_LEVEL.load(core::sync::atomic::Ordering::Relaxed) >= $crate::io::log_macros::LogLevel::Warn as usize {
            $crate::println!("{}[WARN]{} {}", crate::io::colours::YELLOW_FG, crate::io::colours::WHITE_FG, format_args!($($t)*));
        }
    };
}

#[macro_export]
/// Info. Prefix printed in green.
macro_rules! info {
    ($($t:tt)*) => {
        if $crate::io::log_macros::MAX_LOG_LEVEL.load(core::sync::atomic::Ordering::Relaxed) >= $crate::io::log_macros::LogLevel::Info as usize {
            $crate::println!("{}[INFO]{} {}", crate::io::colours::GREEN_FG, crate::io::colours::WHITE_FG, format_args!($($t)*));
        }
    };
}

#[macro_export]
/// Trace. Prefix printed in cyan.
macro_rules! trace {
    ($($t:tt)*) => {
        if $crate::io::log_macros::MAX_LOG_LEVEL.load(core::sync::atomic::Ordering::Relaxed) >= $crate::io::log_macros::LogLevel::Trace as usize {
            $crate::println!("{}[TRACE]{} {}", crate::io::colours::CYAN_FG, crate::io::colours::WHITE_FG, format_args!($($t)*));
        }
    };
}
