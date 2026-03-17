#![no_std]
#![no_main]

#[macro_use]
mod console;

mod alloc;
mod panic;
mod plic;
mod trap;
mod uart;

use core::arch::global_asm;

// entry.S: setup the stack and call main
global_asm!(include_str!("entry.S"));

// trap.S: capture traps and handle exceptions & interrupts
global_asm!(include_str!("trap.S"));

#[unsafe(no_mangle)]
pub extern "C" fn start() -> ! {
    uart::init(); // Serial port — so we can print
    alloc::init(); // Page allocator — so we can allocate memory
    plic::init(); // Interrupt controller — so devices can signal us

    let free_pages = alloc::free_page_count();
    let page = alloc::kalloc();

    println!(
        "Page allocator initialized: {} free pages ({} MB)",
        free_pages,
        (free_pages * alloc::PAGE_SIZE) / (1024 * 1024)
    );

    if page.is_null() {
        println!("kalloc() returned null — allocation failed!");
    } else {
        println!("Allocated page at: {:p}", page);
        alloc::kfree(page);
        println!("Page freed successfully.");
    }

    // This configures the CPU to call our trap_handler when anything
    // goes wrong or when a device needs attention. Three things to set:
    //
    //   mtvec    — "where to jump on a trap" (our trap_vector in trap.S)
    //   mie      — "which interrupt types to accept" (bit 11 = external)
    //   mstatus  — "master interrupt on/off switch" (bit 3 = enable)

    unsafe {
        // Load the address of trap_vector into a register,
        // then write it to mtvec so the CPU knows where to jump on a trap.
        let trap_addr: usize;
        core::arch::asm!("la {}, trap_vector", out(reg) trap_addr);
        core::arch::asm!("csrw mtvec, {}", in(reg) trap_addr);

        // Enable machine external interrupts in mie (Machine Interrupt Enable).
        // Bit 11 = MEIE (Machine External Interrupt Enable).
        // "csrs" = "CSR Set" — sets specific bits without changing others,
        // just like the |= operator you used in the PLIC code.
        core::arch::asm!("csrs mie, {}", in(reg) (1 << 11));

        // Enable global interrupts in mstatus (Machine Status register).
        // Bit 3 = MIE (Machine Interrupt Enable) — the master switch.
        // Even with mie configured above, no interrupts get through
        // until this master switch is flipped on.
        core::arch::asm!("csrs mstatus, {}", in(reg) (1 << 3));
    }

    println!("Waiting for keyboard input...");

    loop {}
}
