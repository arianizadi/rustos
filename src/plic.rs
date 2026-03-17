/// PLIC - Platform-Level Interrupt Controller
///
///   UART (IRQ 10) ──┐
///   Disk (IRQ 1)  ──┼──> PLIC ──> CPU
///   Timer (IRQ 5) ──┘
///
use core::ptr::{read_volatile, write_volatile};

const PLIC_BASE: usize = 0x0C00_0000;
const UART0_IRQ: usize = 10;

/// Each IRQ has a priority register at: PLIC_BASE + (irq * 4)
/// They're laid out sequentially in memory:
///   IRQ 0  → PLIC_BASE + 0
///   IRQ 1  → PLIC_BASE + 4
///   IRQ 2  → PLIC_BASE + 8
///   ...
///   IRQ 10 → PLIC_BASE + 40
fn set_priority(irq: usize, priority: u32) {
    unsafe {
        write_volatile((PLIC_BASE + (irq * 4)) as *mut u32, priority);
    }
}

/// Even with priority set, the PLIC won't forward an interrupt unless
/// it's explicitly enabled.
///
/// The enable register at PLIC_BASE + 0x2000
///   Bit:  31 30 ... 11 10  9 ... 1 0
///          0  0      0  ?  0     0 0
///                       ^
///                       IRQ 10 = UART
///
fn enable_irq(irq: usize) {
    unsafe {
        let mut val = read_volatile((PLIC_BASE + 0x2000) as *mut u32);
        val |= 1 << irq;
        write_volatile((PLIC_BASE + 0x2000) as *mut u32, val);
    }
}

/// Set the priority threshold.
///
/// Only interrupts with priority GREATER than the threshold get through.
/// Threshold register is at: PLIC_BASE + 0x20_0000
///
fn set_threshold(threshold: u32) {
    unsafe {
        write_volatile((PLIC_BASE + 0x0020_0000) as *mut u32, threshold);
    }
}

/// Claim an interrupt
///
/// When the CPU receives an external interrupt, it doesn't know which
/// device caused it. Reading the claim register tells you the IRQ number
/// (e.g., 10 for UART). The PLIC also marks that IRQ as "being handled"
/// so it won't fire again until you call complete().
///
/// Claim register is at: PLIC_BASE + 0x20_0004
pub fn claim() -> u32 {
    unsafe { read_volatile((PLIC_BASE + 0x0020_0004) as *mut u32) }
}

/// After handling an interrupt, you MUST tell the PLIC you're done.
/// Until you do, the PLIC won't send that device's interrupts again.
///
/// Uses the SAME address as claim: PLIC_BASE + 0x20_0004
/// Reading = claim, Writing = complete.
///
pub fn complete(irq: u32) {
    unsafe {
        write_volatile((PLIC_BASE + 0x0020_0004) as *mut u32, irq);
    }
}

/// Initialize the PLIC for UART interrupts.
pub fn init() {
    set_priority(UART0_IRQ, 1);
    enable_irq(UART0_IRQ);
    set_threshold(0);
}
