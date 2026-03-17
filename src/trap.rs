/// Trap Handler
///
/// A "trap" is when the CPU stops what it's doing and jumps to the
/// kernel. This happens for three reasons:
///
///   1. EXCEPTIONS — something went wrong in the running code:
///      - Illegal instruction, bad memory access, divide by zero
///      - The CPU can't continue, so it asks the kernel for help
///
///   2. INTERRUPTS — a hardware device needs attention:
///      - UART received a keypress, timer fired, disk finished reading
///      - The CPU was doing something else but the device can't wait
///
///   3. SYSTEM CALLS (ecall) — user code asks the kernel for a service:
///      - "Read a file", "send a packet", "exit the process"
///
/// How traps flow:
///
///   1. Something triggers a trap
///   2. CPU saves the current PC in mepc ("where was I?")
///   3. CPU writes the cause to mcause ("what happened?")
///   4. CPU jumps to the address in mtvec (our trap_vector in trap.S)
///   5. trap.S saves all 32 registers (so we can restore them later)
///   6. trap.S calls this file's trap_handler()
///   7. We figure out what happened and deal with it
///   8. trap.S restores all registers
///   9. mret jumps back to mepc (resume where we were)
///
/// mcause format:
///   Bit 63 = 1 → interrupt, 0 → exception
///   Bits 62:0  = cause code (what specifically happened)
use core::arch::asm;

use crate::{
    plic,
    uart::{getc, putc},
};

/// Read mcause — tells us WHY the trap happened.
///
/// csrr = "Control and Status Register Read"
///  We can't access CSRs with normal load/store instructions: they have their own instructions.
fn read_mcause() -> usize {
    let val: usize;
    unsafe { asm!("csrr {}, mcause", out(reg) val) };
    val
}

/// Read mepc — tells us WHERE the trap happened.
///
/// For exceptions: this is the instruction that caused the fault.
/// For interrupts: this is the instruction that was interrupted.
/// mret will jump back to this address when the trap handler returns.
fn read_mepc() -> usize {
    let val: usize;
    unsafe { asm!("csrr {}, mepc", out(reg) val) };
    val
}

/// Main trap handler: called from trap.S after registers are saved.
#[unsafe(no_mangle)]
pub extern "C" fn trap_handler() {
    let mcause = read_mcause();
    let mepc = read_mepc();

    // Extract the cause code. Bit 63 indicates interrupt vs exception,
    // so we mask it off to get just the cause number.
    // 0x7fff_ffff_ffff_ffff = all bits except bit 63
    let cause = mcause & 0x7fff_ffff_ffff_ffff;

    if mcause >> 63 == 1 {
        // ── INTERRUPT ──────────────────────────────────────────────
        // The CPU was interrupted by an external event.
        // Common M-mode interrupt cause codes:
        //   3  = machine software interrupt
        //   7  = machine timer interrupt
        //   11 = machine external interrupt (from PLIC: a device)
        match cause {
            11 => {
                // External interrupt: some device signaled via the PLIC.
                // We need to ask the PLIC which device it was.
                let irq = plic::claim();
                if irq == 10 {
                    // IRQ 10 = UART: a key was pressed.
                    // Read the byte and echo it back to the terminal.
                    if let Some(byte) = getc() {
                        putc(byte);
                    }
                }
                // Tell the PLIC we're done handling this interrupt.
                // Until we do this, the PLIC won't send this IRQ again.
                plic::complete(irq);
            }
            _ => {
                println!("Unknown interrupt: cause={cause}");
            }
        }
    } else {
        // ── EXCEPTION ──────────────────────────────────────────────
        // Something went wrong in the running code.
        match cause {
            0 => {
                println!("Instruction address misaligned");
            }
            2 => {
                println!("Illegal instruction");
            }
            5 => {
                println!("Load access fault");
            }
            7 => {
                println!("Store access fault");
            }
            _ => {
                println!("Unknown exception: cause={cause}, mepc=0x{mepc:x}");
            }
        }

        // Advance mepc past the faulting instruction (4 bytes = 1 instruction).
        // Without this, mret would jump back to the SAME instruction that
        // caused the fault, creating an infinite loop of exceptions.
        //
        // In a real OS with virtual memory, page fault handlers would NOT
        // skip — they'd map the missing page and re-execute the instruction.
        unsafe {
            asm!("csrr {0}, mepc", "addi {0}, {0}, 4", "csrw mepc, {0}", out(reg) _);
        }
    }
}
