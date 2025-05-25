//! Types related to task management

use super::TaskContext;
use crate::config::{TRAP_CONTEXT, kernel_stack_position};
use crate::mm::{KERNEL_SPACE, MapPermission, MemorySet, PhysPageNum, VirtAddr};
use crate::trap::{TrapContext, trap_handler};

/// task control block structure
pub struct TaskControlBlock {
    /// task status: Ready/Running/Exited
    pub task_status: TaskStatus,
    /// task context
    pub task_cx: TaskContext,
    /// memory set
    pub memory_set: MemorySet,
    /// trap context physical page number
    pub trap_cx_ppn: PhysPageNum,
    /// base size
    #[allow(unused)]
    pub base_size: usize,
    /// heap bottom
    pub heap_bottom: usize,
    /// program break
    pub program_brk: usize,
    /// syscall count array
    pub syscall_count: [usize; 500],
}

impl TaskControlBlock {
    /// Get trap context
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    /// Get user token
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

    /// Create a new task control block
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;
        // map a kernel-stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        let task_control_block = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
            heap_bottom: user_sp,
            program_brk: user_sp,
            syscall_count: [0; 500],
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }

    /// change the location of the program break. return None if failed.
    pub fn change_program_brk(&mut self, size: i32) -> Option<usize> {
        let old_break = self.program_brk;
        let new_brk = self.program_brk as isize + size as isize;
        if new_brk < self.heap_bottom as isize {
            return None;
        }
        let result = if size < 0 {
            self.memory_set
                .shrink_to(VirtAddr(self.heap_bottom), VirtAddr(new_brk as usize))
        } else {
            self.memory_set
                .append_to(VirtAddr(self.heap_bottom), VirtAddr(new_brk as usize))
        };
        if result {
            self.program_brk = new_brk as usize;
            Some(old_break)
        } else {
            None
        }
    }

    /// mmap system call implementation
    /// 参考资料: rCore Tutorial Book Chapter 4 - 匿名内存映射实现
    /// 参考资料: RISC-V Privileged Architecture Specification v1.20 - 页表权限位设置
    pub fn mmap(&mut self, start: usize, len: usize, prot: usize) -> isize {
        use crate::config::PAGE_SIZE;

        // Check if start is page-aligned
        // 参考资料: rCore Tutorial Book Chapter 4 - 页边界对齐检查
        if start % PAGE_SIZE != 0 {
            return -1;
        }

        // Check if prot is valid
        // 参考资料: RISC-V Privileged Architecture Specification v1.20 - 页表权限验证
        if prot & !0x7 != 0 || prot & 0x7 == 0 {
            return -1;
        }

        // Convert len to page-aligned
        let len = (len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        if len == 0 {
            return 0;
        }

        // Convert prot to MapPermission
        // 参考资料: rCore Tutorial Book Chapter 4 - 权限位转换机制
        let mut map_perm = MapPermission::U;
        if prot & 0x1 != 0 { map_perm |= MapPermission::R; }
        if prot & 0x2 != 0 { map_perm |= MapPermission::W; }
        if prot & 0x4 != 0 { map_perm |= MapPermission::X; }

        // Try to map the memory
        self.memory_set.mmap(VirtAddr(start), len, map_perm)
    }

    /// munmap system call implementation
    /// 参考资料: rCore Tutorial Book Chapter 4 - 内存解映射和页表清理
    pub fn munmap(&mut self, start: usize, len: usize) -> isize {
        use crate::config::PAGE_SIZE;

        // Check if start is page-aligned
        // 参考资料: rCore Tutorial Book Chapter 4 - 页边界对齐检查
        if start % PAGE_SIZE != 0 {
            return -1;
        }

        // Convert len to page-aligned
        let len = (len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        if len == 0 {
            return 0;
        }

        // Try to unmap the memory
        self.memory_set.munmap(VirtAddr(start), len)
    }

    /// trace system call implementation
    /// 参考资料: rCore Tutorial Book Chapter 4 - 内存跟踪和调试功能
    /// 参考资料: RISC-V Privileged Architecture Specification v1.20 - 页表权限检查机制
    pub fn trace(&mut self, addr: usize, trace_request: usize, data: usize) -> isize {
        use crate::mm::VirtAddr;

        match trace_request {
            0 => {
                // Read operation
                let vpn = VirtAddr(addr).floor();

                // Check if the page is mapped and get permissions
                if let Some(pte) = self.memory_set.translate(vpn) {
                    if !pte.is_valid() {
                        return -1;
                    }

                    // Check if readable and user accessible
                    if !pte.readable() || !pte.user() {
                        return -1;
                    }

                    // Read the byte at the address
                    let ppn = pte.ppn();
                    let offset = VirtAddr(addr).page_offset();
                    let byte_array = ppn.get_bytes_array();
                    byte_array[offset] as isize
                } else {
                    -1
                }
            }
            1 => {
                // Write operation
                let vpn = VirtAddr(addr).floor();

                // Check if the page is mapped and get permissions
                if let Some(pte) = self.memory_set.translate(vpn) {
                    if !pte.is_valid() {
                        return -1;
                    }

                    // Check if writable and user accessible
                    if !pte.writable() || !pte.user() {
                        return -1;
                    }

                    // Write the byte to the address
                    let ppn = pte.ppn();
                    let offset = VirtAddr(addr).page_offset();
                    let byte_array = ppn.get_bytes_array();
                    byte_array[offset] = data as u8;
                    0
                } else {
                    -1
                }
            }
            2 => {
                // Syscall count operation
                if addr < 500 {
                    self.syscall_count[addr] as isize
                } else {
                    -1
                }
            }
            _ => -1,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
