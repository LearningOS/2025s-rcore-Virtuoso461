#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::mmap;

#[no_mangle]
fn main() -> i32 {
    let start: usize = 0x10000000;
    let len: usize = 4096;
    let prot: usize = 3;
    
    println!("Testing mmap...");
    
    // First mmap should succeed
    let result1 = mmap(start, len, prot);
    println!("mmap(0x{:x}, {}, {}) = {}", start, len, prot, result1);
    
    // Test overlapping mmap
    let result2 = mmap(start - len, len + 1, prot);
    println!("mmap(0x{:x}, {}, {}) = {}", start - len, len + 1, prot, result2);
    
    // Test adjacent mmap
    let result3 = mmap(start + len + 1, len, prot);
    println!("mmap(0x{:x}, {}, {}) = {}", start + len + 1, len, prot, result3);
    
    // Test invalid prot
    let result4 = mmap(start + len, len, 0);
    println!("mmap(0x{:x}, {}, {}) = {}", start + len, len, 0, result4);
    
    let result5 = mmap(start + len, len, prot | 8);
    println!("mmap(0x{:x}, {}, {}) = {}", start + len, len, prot | 8, result5);
    
    0
}
