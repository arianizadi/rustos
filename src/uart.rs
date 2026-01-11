use core::fmt;
use core::ptr::{read_volatile, write_volatile};

// --- LOW LEVEL DRIVERS ---

const UART_BASE: usize = 0x1000_0000;

pub fn init() {
    unsafe {
        /*
         * LCR - Line Control Register
         * 0x3 = 0b0000_0011
         *
         * Bit 0,1 - Setting word length
         *
         * 0,0 => 5 Bits
         * 0,1 => 6 Bits
         * 1,0 => 7 Bits
         * 1,1 => 8 Bits
         *
         * We want 8 bit words because ASCII is 8 bit.
         */
        write_volatile((UART_BASE + 0x3) as *mut u8, 0x03);

        /*
         * FCR - First In / First Out Control Register
         * 0x1 = 0b0000_0001
         *
         * Bit 0 = Enable FIFO
         *
         * We want to enable the buffer to store words before the current
         * is shifted out.
         */
        write_volatile((UART_BASE + 0x2) as *mut u8, 0x01);
    }
}

pub fn putc(c: u8) {
    unsafe {
        /*
         * LSR Register - Line Status Register
         * Bit 5 - Empty Transmitter Holding Register
         *
         * We are interested in Bit 5, if register is empty we can write.
         */
        while read_volatile((UART_BASE + 0x5) as *mut u8) & 0b0010_0000 == 0 {}
        write_volatile(UART_BASE as *mut u8, c);
    }
}

// --- HIGH LEVEL DRIVERS ---
pub struct UartWriter;

impl fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            putc(c);
        }
        Ok(())
    }
}
