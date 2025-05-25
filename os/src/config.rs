//! Constants used in rCore

/// user app's stack size
pub const USER_STACK_SIZE: usize = 4096 * 2;
/// kernel heap size
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
/// kernel stack size
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
/// page size
pub const PAGE_SIZE: usize = 0x1000;
/// page size bits
pub const PAGE_SIZE_BITS: usize = 0xc;

/// physical memory end
pub const MEMORY_END: usize = 0x88000000;
/// the max number of apps
pub const MAX_APP_NUM: usize = 16;
/// the max number of syscalls
pub const MAX_SYSCALL_NUM: usize = 500;

/// clock frequency
pub const CLOCK_FREQ: usize = 12500000;

/// trampoline virtual address
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
/// trap context virtual address
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
