//! Parsing of the kernel command line.

use core::ffi::CStr;

use crate::{io::log_macros, trace, warn, KERNEL_FILE};

/// Parse the kernel command line
pub fn get_cmdline_vars() {
    // get the kernel file from the bootloader
    let kernel_file_resp = KERNEL_FILE
        .get_response()
        .get()
        .expect("Bootloader did not respond to kernel file request.");
    // take the bootloader response and turn it into a reference to the kernel file
    // !SAFETY: This is safe because the bootloader will only return a valid pointer to the kernel.
    let kernel_file = unsafe { &*(kernel_file_resp.kernel_file.as_ptr().unwrap()) };
    // !SAFETY: The bootloader cmdline is a null-terminated string, so this is safe. It is also static.
    let cmdline = unsafe {
        CStr::from_ptr(kernel_file.cmdline.as_ptr().unwrap())
            .to_str()
            .unwrap()
    };
    trace!("Command line: {}", cmdline);
    parse_cmdline(cmdline);
}

// TODO: This should not deal with what the options are, but rather just parse them.
// This means we also need to find some way to pass the parsed options to the rest of the kernel.
fn parse_cmdline(cmdline: &str) {
    // get each key=value pair
    let args = cmdline.split_whitespace();
    // split each pair into a tuple of (key, value)
    let args = args.map(|arg| {
        let mut split = arg.splitn(2, '=');
        let key = split.next().unwrap();
        let value = split.next().unwrap_or("");
        (key, value)
    });
    // look through each argument and set the appropriate option (again, this should not be here)
    for arg in args {
        if arg.0 == "LOG_LEVEL" {
            match arg.1 {
                "TRACE" => log_macros::set_log_level(log_macros::LogLevel::Trace),
                "INFO" => log_macros::set_log_level(log_macros::LogLevel::Info),
                "WARN" => log_macros::set_log_level(log_macros::LogLevel::Warn),
                "ERROR" => log_macros::set_log_level(log_macros::LogLevel::Error),
                "FATAL" => log_macros::set_log_level(log_macros::LogLevel::Fatal),
                _ => warn!("Invalid log level: {}", arg.1),
            }
        }
    }
}
