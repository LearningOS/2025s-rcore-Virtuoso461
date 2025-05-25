//! SBI call wrappers

use core::arch::asm;

// SBI constants for QEMU 7 compatibility
const SBI_SET_TIMER: usize = 0x0;
const SBI_CONSOLE_PUTCHAR: usize = 0x01;
const SBI_CONSOLE_GETCHAR: usize = 0x02;
const SBI_SHUTDOWN: usize = 0x08;

#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") which,
        );
    }
    ret
}

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

/// use sbi call to getchar from console (qemu uart handler)
pub fn console_getchar() -> usize {
    sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0)
}

/// use sbi call to set timer
pub fn set_timer(timer: usize) {
    sbi_call(SBI_SET_TIMER, timer, 0, 0);
}

/// use sbi call to shutdown the kernel
pub fn shutdown(failure: bool) -> ! {
    let fail = if failure { 1 } else { 0 };
    sbi_call(SBI_SHUTDOWN, fail, 0, 0);
    unreachable!()
}
