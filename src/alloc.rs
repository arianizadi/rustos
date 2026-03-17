/// Page Allocator — manages physical memory using a free list.
///
///   FREE_LIST
///      │
///      ▼
///   ┌────────┐     ┌────────┐     ┌────────┐
///   │ next ──┼────►│ next ──┼────►│ null   │
///   │  4KB   │     │  4KB   │     │  4KB   │
///   └────────┘     └────────┘     └────────┘
///
///
/// Memory layout (QEMU virt machine):
///   0x80000000 ──────────────────────── 0x88000000
///   │  kernel  │     free pages...     │
///   └──────────┴───────────────────────┘
///              ↑                       ↑
///      linker `end` symbol       PHYS_END (128MB)
use core::ptr;

/// 4KB — the standard page size, matching RISC-V's virtual memory
pub const PAGE_SIZE: usize = 4096;

/// Top of physical RAM on QEMU virt machine.
/// 0x80000000 (start) + 0x08000000 (128MB) = 0x88000000
const PHYS_END: usize = 0x8800_0000;

struct FreeNode {
    next: *mut FreeNode,
}

/// Head of the free list. Points to the first free page.
/// null = no free pages left (out of memory).
static mut FREE_LIST: *mut FreeNode = ptr::null_mut();

/// Build the free list by freeing every page from kernel end to RAM end.
pub fn init() {
    unsafe extern "C" {
        static end: u8;
    }

    let heap_start = {
        let end_addr = &raw const end as usize;
        // Align to 4kb
        (end_addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
    };

    // Walk through all pages and add each one to the free list.
    let mut addr = heap_start;
    while addr + PAGE_SIZE <= PHYS_END {
        kfree(addr as *mut u8);
        addr += PAGE_SIZE;
    }
}

/// Free a page: push it onto the front of the free list.
pub fn kfree(page: *mut u8) {
    unsafe {
        let node = page as *mut FreeNode;
        (*node).next = FREE_LIST;
        FREE_LIST = node;
    }
}

/// Allocate a 4KB page: pop from the front of the free list.
/// Returns null if no free pages remain.
pub fn kalloc() -> *mut u8 {
    unsafe {
        let node = FREE_LIST;
        if !node.is_null() {
            FREE_LIST = (*node).next;
        }
        node as *mut u8
    }
}

/// Count free pages by walking the entire list.
pub fn free_page_count() -> usize {
    let mut count = 0;
    unsafe {
        let mut node = FREE_LIST;
        while !node.is_null() {
            count += 1;
            node = (*node).next;
        }
    }
    count
}
