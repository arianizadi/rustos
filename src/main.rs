#![no_std]
#![no_main]

mod panic;
mod uart;

use core::arch::global_asm;

use crate::uart::print_str;

global_asm!(include_str!("entry.S"));

#[unsafe(no_mangle)]
pub extern "C" fn start() -> ! {
    uart::init();
    print_str("Hello, World!\n");
    loop {}
}
