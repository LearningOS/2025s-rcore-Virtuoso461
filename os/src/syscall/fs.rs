//! File system related syscalls

use crate::fs::{open_file, OpenFlags, Stat, link_file, unlink_file};
use crate::mm::{translated_str, translated_refmut, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        let token = current_user_token();
        let user_buf = UserBuffer::new(translated_byte_buffer(token, buf, len));
        file.read(user_buf) as isize
    } else {
        -1
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        let token = current_user_token();
        let user_buf = UserBuffer::new(translated_byte_buffer(token, buf, len));
        file.write(user_buf) as isize
    } else {
        -1
    }
}

pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    let task = current_task().unwrap();

    // First get the file reference without holding the lock
    let file = {
        let inner = task.inner_exclusive_access();
        if fd >= inner.fd_table.len() {
            return -1;
        }
        if let Some(file) = &inner.fd_table[fd] {
            file.clone()
        } else {
            return -1;
        }
    };

    // Now call fstat without holding the task lock
    let stat = file.fstat();
    let token = current_user_token();
    *translated_refmut(token, st) = stat;
    0
}

pub fn sys_linkat(
    _olddirfd: i32,
    oldpath: *const u8,
    _newdirfd: i32,
    newpath: *const u8,
    _flags: u32,
) -> isize {
    let token = current_user_token();
    let oldpath = translated_str(token, oldpath);
    let newpath = translated_str(token, newpath);

    // Check if trying to link to the same name
    if oldpath == newpath {
        return -1;
    }

    if link_file(newpath.as_str(), oldpath.as_str()) {
        0
    } else {
        -1
    }
}

pub fn sys_unlinkat(_dirfd: i32, path: *const u8, _flags: u32) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if unlink_file(path.as_str()) {
        0
    } else {
        -1
    }
}

use crate::mm::translated_byte_buffer;
