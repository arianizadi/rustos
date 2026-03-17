/*
   Boot Code - First code to run on power on

   1. Set up a stack pointer so function calls work
   2. Jump to Rust code (start() in main.rs)

   RISC-V chips have multiple "harts" (hardware threads).
   Each hart needs its own stack so they don't overwrite each other.

     stack0 (base addr)
     +-- Hart 0 stack: 4KB  (stack0 + 4096*1)
     +-- Hart 1 stack: 4KB  (stack0 + 4096*2)
     +-- ...
     +-- Hart 7 stack: 4KB  (stack0 + 4096*8)

   Stacks grow DOWNWARD in RISC-V, so each hart's SP points to
   the TOP of its stack region.
*/

.section .text.entry
.global _entry
_entry:
        la sp, stack0
        li a0, 1024*4
        csrr a1, mhartid
        addi a1, a1, 1
        mul a0, a0, a1
        add sp, sp, a0
        call start
spin:
        j spin
