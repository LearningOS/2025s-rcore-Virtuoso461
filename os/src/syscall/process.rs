//! Process management syscalls

use crate::mm::translated_refmut;
use crate::task::{change_program_brk, current_mmap, current_munmap, current_trace, current_user_token, exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::get_time_ms;
use super::TimeVal;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

/// get current time
pub fn sys_get_time(time: *mut TimeVal, _tz: usize) -> isize {
    let time_ms = get_time_ms();
    let token = current_user_token();
    let time_ref = translated_refmut(token, time);
    time_ref.sec = time_ms / 1000;
    time_ref.usec = (time_ms % 1000) * 1000;
    0
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// mmap system call implementation
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    current_mmap(start, len, prot)
}

/// munmap system call implementation
pub fn sys_munmap(start: usize, len: usize) -> isize {
    current_munmap(start, len)
}

/// trace system call implementation
pub fn sys_trace(trace_request: usize, addr: usize, data: usize) -> isize {
    current_trace(addr, trace_request, data)
}
