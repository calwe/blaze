//! ANSI colour escape codes

#![allow(dead_code)]

#[allow(missing_docs)]
pub const BLACK_FG: &str = "\x1b[30m";
#[allow(missing_docs)]
pub const RED_FG: &str = "\x1b[31m";
#[allow(missing_docs)]
pub const GREEN_FG: &str = "\x1b[32m";
#[allow(missing_docs)]
pub const YELLOW_FG: &str = "\x1b[33m";
#[allow(missing_docs)]
pub const BLUE_FG: &str = "\x1b[34m";
#[allow(missing_docs)]
pub const MAGENTA_FG: &str = "\x1b[35m";
#[allow(missing_docs)]
pub const CYAN_FG: &str = "\x1b[36m";
#[allow(missing_docs)]
pub const WHITE_FG: &str = "\x1b[37m";

#[allow(missing_docs)]
pub const BLACK_BG: &str = "\x1b[40m";
#[allow(missing_docs)]
pub const RED_BG: &str = "\x1b[41m";
#[allow(missing_docs)]
pub const GREEN_BG: &str = "\x1b[42m";
#[allow(missing_docs)]
pub const YELLOW_BG: &str = "\x1b[43m";
#[allow(missing_docs)]
pub const BLUE_BG: &str = "\x1b[44m";
#[allow(missing_docs)]
pub const MAGENTA_BG: &str = "\x1b[45m";
#[allow(missing_docs)]
pub const CYAN_BG: &str = "\x1b[46m";
#[allow(missing_docs)]
pub const WHITE_BG: &str = "\x1b[47m";
