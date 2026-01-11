#![no_std]
#![no_main]

#[macro_use]
mod console;

mod panic;
mod uart;

use core::arch::global_asm;

global_asm!(include_str!("entry.S"));

#[unsafe(no_mangle)]
pub extern "C" fn start() -> ! {
    uart::init();
    println!("Hello, world!");
    let x = 5;
    let y = 3;
    println!("Math test: {} + {} = {}", x, y, x + y);
    loop {}
}
