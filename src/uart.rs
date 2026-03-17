/// UART (Universal Asynchronous Receiver/Transmitter)
///   Address          Register   Purpose
///   ─────────────────────────────────────────────────────────
///   UART_BASE + 0x0  THR/RBR    Transmit (write) / Receive (read)
///   UART_BASE + 0x1  IER        Interrupt Enable Register
///   UART_BASE + 0x2  FCR        FIFO Control Register
///   UART_BASE + 0x3  LCR        Line Control Register
///   UART_BASE + 0x5  LSR        Line Status Register
///
/// All registers are accessed via memory-mapped I/O, so no ram just straight to hardware
use core::fmt;
use core::ptr::{read_volatile, write_volatile};

const UART_BASE: usize = 0x1000_0000;

/// Initialize the UART hardware.
///
///   1. LCR — set 8-bit word length (needed for ASCII)
///   2. IER — enable receive interrupts (so we get notified on keypress)
///   3. FCR — enable FIFO buffer (so bytes queue up instead of getting lost)
pub fn init() {
    unsafe {
        // LCR (Line Control Register) — offset 0x3
        // 0x03 = 0b0000_0011 → bits 0,1 set = 8-bit word length
        // We need 8-bit words because ASCII characters are 8 bits.
        write_volatile((UART_BASE + 0x3) as *mut u8, 0x03);

        // IER (Interrupt Enable Register) — offset 0x1
        // 0x01 = 0b0000_0001 → bit 0 set = "Received Data Available" interrupt
        // This tells the UART: "interrupt the CPU when a byte arrives."
        // Without this, we'd have to constantly look for input.
        write_volatile((UART_BASE + 0x1) as *mut u8, 0x01);

        // FCR (FIFO Control Register) — offset 0x2
        // 0x01 = 0b0000_0001 → bit 0 set = enable FIFO
        // The FIFO is a small hardware buffer that holds incoming/outgoing
        // bytes. Without it, if a second byte arrives before we read the
        // first, the first byte is lost.
        write_volatile((UART_BASE + 0x2) as *mut u8, 0x01);
    }
}

/// Send one byte out the serial port.
pub fn putc(c: u8) {
    unsafe {
        // LSR (Line Status Register) — offset 0x5
        // Bit 5 = THR Empty (Transmitter Holding Register Empty)
        // When bit 5 is 0, the transmitter is busy we spin and wait.
        // When bit 5 is 1, we can safely write a byte.
        while read_volatile((UART_BASE + 0x5) as *mut u8) & 0b0010_0000 == 0 {}

        // Write the byte to THR (Transmit Holding Register): offset 0x0
        // The UART hardware shifts this byte out the serial line.
        write_volatile(UART_BASE as *mut u8, c);
    }
}

/// Try to read one byte from the serial port (non-blocking).
///
/// Returns Some(byte) if a character was received
/// or None if the receive buffer is empty.
pub fn getc() -> Option<u8> {
    unsafe {
        // LSR (Line Status Register) — offset 0x5
        // Bit 0 = Data Ready: 1 means a byte is waiting to be read.
        if read_volatile((UART_BASE + 0x5) as *mut u8) & 0b0000_0001 == 0 {
            None // No data available
        } else {
            // Read from RBR (Receive Buffer Register): offset 0x0
            // Same address as THR, but reading vs writing accesses
            // different hardware registers.
            Some(read_volatile(UART_BASE as *mut u8))
        }
    }
}

pub struct UartWriter;

impl fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            putc(c);
        }
        Ok(())
    }
}
