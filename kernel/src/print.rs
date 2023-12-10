use core::fmt;
use spin::Mutex;

use crate::util::outb;

pub struct Writer;

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            put_char(c);
        }
        Ok(())
    }
}

static WRITER: Mutex<Writer> = Mutex::new(Writer);

pub fn _print(args: fmt::Arguments) {
    let mut writer = WRITER.lock();
    let _ = fmt::Write::write_fmt(&mut *writer, args);
}

pub fn put_char(c: char) {
    if !c.is_ascii() {
        return;
    }

    unsafe { 
        outb(0xe9, c as u8);
    }
}


#[macro_export]
macro_rules! print {
    ($($t:tt)*) => { $crate::print::_print(format_args!($($t)*)) };
}

#[macro_export]
macro_rules! println {
    ()          => { $crate::print!("\n"); };
    ($($t:tt)*) => { $crate::print!("{}\n", format_args!($($t)*)); };
}
