use core::ptr::{read_volatile, write_volatile};

const UART_BASE: usize = 0x1000_0000;

pub fn init() {
    unsafe {
        write_volatile((UART_BASE + 0x3) as *mut u8, 0x03);
        write_volatile((UART_BASE + 0x2) as *mut u8, 0x01);
    }
}

pub fn putc(c: u8) {
    unsafe {
        /*
         * LSR Register - Line Status Register
         * Bit 0 - Data Ready
         * Bit 1 - Overrun Error
         * Bit 2 - Parity Error
         * Bit 3 - Framing Error
         * Bit 4 - Break Interrupt
         * Bit 5 - Empty Transmitter Holding Register
         * Bit 6 - Empty Data Holding Registers
         * Bit 7 - Error in Recieved FIFO
         *
         * We are interested in Bit 5, if register is empty we can write.
         * Bit 6 is used for an empty holding register and shift register
         * but since we can use pipelining, we don't have to wait for shift.
         */
        while read_volatile((UART_BASE + 0x5) as *mut u8) & 0b0010_0000 == 0 {}
        write_volatile(UART_BASE as *mut u8, c);
    }
}
