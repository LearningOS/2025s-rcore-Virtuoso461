//! File system implementation
//!
//! This module provides file system abstraction and implementation.

mod inode;
mod stdio;

use crate::mm::UserBuffer;

pub use inode::{list_apps, open_file, OpenFlags, OSInode, link_file, unlink_file};
pub use stdio::{Stdin, Stdout};

/// trait File for all file types
pub trait File: Send + Sync {
    /// the file readable?
    fn readable(&self) -> bool;
    /// the file writable?
    fn writable(&self) -> bool;
    /// read from the file to buf, return the number of bytes read
    fn read(&self, buf: UserBuffer) -> usize;
    /// write to the file from buf, return the number of bytes written
    fn write(&self, buf: UserBuffer) -> usize;
    /// get file stat
    fn fstat(&self) -> Stat;
    /// for downcasting
    fn as_any(&self) -> &dyn core::any::Any;
}

/// Stat structure for file information
#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    /// device id
    pub dev: u64,
    /// inode number
    pub ino: u64,
    /// file type and mode
    pub mode: StatMode,
    /// number of hard links
    pub nlink: u32,
    /// padding
    pub pad: [u64; 7],
}

bitflags! {
    /// File type and mode
    pub struct StatMode: u32 {
        /// null
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// regular file
        const FILE  = 0o100000;
    }
}
