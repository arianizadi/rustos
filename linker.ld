/*
 * Linker Script — tells the linker how to lay out the kernel in memory.
 *
 * When you compile Rust code, the compiler produces object files with
 * "sections" (.text for code, .data for globals, etc.). The linker
 * combines them into one binary. This script controls WHERE each
 * section ends up in physical memory.
 *
 * RISC-V convention: RAM starts at 0x80000000 on the QEMU virt machine.
 * The kernel is loaded at this address, so our code must be linked there.
 *
 * Memory layout after linking:
 *
 *   0x80000000  ┌──────────────┐
 *               │ .text        │  Code (executable instructions)
 *               │ (page aligned)│
 *               ├──────────────┤
 *               │ .rodata      │  Read-only data (string literals, consts)
 *               ├──────────────┤
 *               │ .data        │  Initialized mutable globals
 *               ├──────────────┤
 *               │ .bss         │  Uninitialized globals (zeroed at boot)
 *               ├──────────────┤
 *               │ .stack       │  Kernel stack (32KB for up to 8 harts)
 *               ├──────────────┤
 *               │ end ←        │  Symbol marking end of kernel
 *               │              │  Everything after this is free RAM
 *   0x88000000  └──────────────┘  (used by the page allocator)
 */

OUTPUT_ARCH( "riscv" )

/* Entry point — the linker ensures _entry (from entry.S) is the first
 * thing at 0x80000000, so the CPU executes it on boot. */
ENTRY( _start )

SECTIONS
{
  /* Start placing sections at 0x80000000 (beginning of RAM). */
  . = 0x80000000;

  .text : {
    /* KEEP ensures entry.S isn't removed by linker garbage collection.
     * .text.entry is placed FIRST so _entry is at exactly 0x80000000. */
    KEEP(*(.text.entry))
    *(.text .text.*)
    /* Align to page boundary (4KB) — needed for virtual memory later,
     * where code pages and data pages need different permissions. */
    . = ALIGN(0x1000);
  }

  .rodata : {
    . = ALIGN(16);
    *(.srodata .srodata.*)
    . = ALIGN(16);
    *(.rodata .rodata.*)
  }

  .data : {
    . = ALIGN(16);
    *(.sdata .sdata.*)
    . = ALIGN(16);
    *(.data .data.*)
  }

  /* BSS = "Block Started by Symbol" — uninitialized global variables.
   * These are guaranteed to be zeroed. The compiler puts globals with
   * no initial value here to save space in the binary (no need to
   * store a bunch of zeros). */
  .bss : {
    . = ALIGN(16);
    *(.sbss .sbss.*)
    . = ALIGN(16);
    *(.bss .bss.*)
  }

  /* Kernel stack — 4KB per hart (CPU core), 8 harts max = 32KB total.
   * stack0 marks the BOTTOM of the stack region. entry.S calculates
   * each hart's stack pointer as: stack0 + 4096 * (hart_id + 1)
   * (Stacks grow downward in RISC-V.) */
  .stack : {
    . = ALIGN(16);
    stack0 = .;
    . += 4096 * 8;
  }

  /* PROVIDE(end) creates a symbol at the current address — right after
   * the last section. The page allocator (alloc.rs) uses this to know
   * where free RAM begins. Everything from `end` to 0x88000000 is
   * available for allocation. */
  PROVIDE(end = .);
}
