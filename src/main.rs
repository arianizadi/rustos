#![no_std]
#![no_main]

mod panic;
mod uart;

use core::arch::global_asm;

global_asm!(include_str!("entry.S"));

#[unsafe(no_mangle)]
pub extern "C" fn start() -> ! {
    uart::init();
    uart::putc(b'H');
    uart::putc(b'e');
    uart::putc(b'l');
    uart::putc(b'l');
    uart::putc(b'o');
    uart::putc(b'\n');
    loop {}
}
