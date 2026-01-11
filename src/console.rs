use crate::uart::UartWriter;
use core::fmt::{self, Write};

pub fn _print(args: fmt::Arguments) {
    let mut writer = UartWriter;
    writer.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {$crate::print!("\n")};
    ($($arg:tt)*) => {$crate::print!("{}\n", format_args!($($arg)*))};
}
