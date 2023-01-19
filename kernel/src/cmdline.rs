use core::ffi::CStr;

use crate::{io::log_macros, trace, warn, KERNEL_FILE};

pub fn get_cmdline_vars() {
    let kernel_file_resp = KERNEL_FILE
        .get_response()
        .get()
        .expect("Bootloader did not respond to kernel file request.");
    let kernel_file = unsafe { &*(kernel_file_resp.kernel_file.as_ptr().unwrap()) };
    let cmdline = unsafe {
        CStr::from_ptr(kernel_file.cmdline.as_ptr().unwrap())
            .to_str()
            .unwrap()
    };
    trace!("Command line: {}", cmdline);
    parse_cmdline(cmdline);
}

fn parse_cmdline(cmdline: &str) {
    let args = cmdline.split_whitespace();
    let args = args.map(|arg| {
        let mut split = arg.splitn(2, '=');
        let key = split.next().unwrap();
        let value = split.next().unwrap_or("");
        (key, value)
    });
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
