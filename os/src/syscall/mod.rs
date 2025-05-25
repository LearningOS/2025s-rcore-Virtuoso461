//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GETTIMEOFDAY: usize = 169;
const SYSCALL_SBRK: usize = 214;
const SYSCALL_MUNMAP: usize = 215;
const SYSCALL_MMAP: usize = 222;
const SYSCALL_TRACE: usize = 410;

mod fs;
mod process;

use fs::*;
use process::*;

/// Time value structure for gettimeofday syscall
#[repr(C)]
#[derive(Debug, Default)]
pub struct TimeVal {
    /// seconds
    pub sec: usize,
    /// microseconds
    pub usec: usize,
}

/// handle syscall exception with `syscall_id` and other arguments
/// 参考资料: rCore Tutorial Book Chapter 4 - 系统调用处理机制
/// 参考资料: RISC-V Assembly Programmer's Manual - ecall指令和系统调用约定
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        // 参考资料: rCore Tutorial Book Chapter 4 - 时间获取系统调用实现
        SYSCALL_GETTIMEOFDAY => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_SBRK => sys_sbrk(args[0] as i32),
        // 参考资料: rCore Tutorial Book Chapter 4 - 内存映射系统调用实现
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        // 参考资料: rCore Tutorial Book Chapter 4 - 内存跟踪系统调用实现
        SYSCALL_TRACE => sys_trace(args[0], args[1], args[2]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
