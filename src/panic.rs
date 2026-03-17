/// Panic Handler
///   1. Prints what went wrong (so we can debug)
///   2. Halts the CPU (there's no safe way to continue)
///
/// This gets called at these cases
///   - unwrap() on None/Err
///   - array out-of-bounds access
///   - explicit panic!("message")
///   - assert! fail
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop {}
}
