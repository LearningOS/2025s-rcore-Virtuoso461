//! SBI call wrappers
//!
//! SBI版本兼容性问题的解决参考了微信群中助教殷老师、群友Brad、TomZz、王然、baitwo02等的建议
//! 参考资料：
//! 1. RISC-V SBI Specification v1.0 - 了解SBI调用规范和常量定义
//! 2. rCore Tutorial Book Chapter 1-2 - 理解SBI接口的使用方法
//! 3. RustSBI文档 - 了解rustsbi-qemu的版本差异和兼容性问题

use core::arch::asm;

// 使用新版SBI规范的常量定义，解决了与旧版本的兼容性问题
// 参考：RISC-V SBI Specification v1.0, Table 4.1 - SBI Extension IDs
const SBI_SET_TIMER: usize = 0x54494D45;  // "TIME" in ASCII
const SBI_CONSOLE_PUTCHAR: usize = 1;     // Legacy console putchar
const SBI_SHUTDOWN: usize = 0x53525354;   // "SRST" in ASCII

/// general sbi call
/// 参考：RISC-V SBI Specification v1.0, Section 3 - SBI Calling Convention
/// 使用标准的RISC-V调用约定进行SBI调用
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        // 参考：RISC-V Assembly Programmer's Manual - ecall指令和寄存器约定
        asm!(
            "ecall",                    // 环境调用指令，触发异常进入S态
            inlateout("x10") arg0 => ret,  // a0: 参数和返回值
            in("x11") arg1,             // a1: 第二个参数
            in("x12") arg2,             // a2: 第三个参数
            in("x16") 0,                // a6: SBI函数ID (0表示使用legacy接口)
            in("x17") which,            // a7: SBI扩展ID
        );
    }
    ret
}

/// use sbi call to set timer
pub fn set_timer(timer: usize) {
    sbi_call(SBI_SET_TIMER, timer, 0, 0);
}

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

/// use sbi call to shutdown the kernel
pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}
