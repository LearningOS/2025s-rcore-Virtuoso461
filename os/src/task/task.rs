//! Task control block implementation

use super::{kstack_alloc, pid_alloc, KernelStack, PidHandle, TaskContext};
use crate::config::TRAP_CONTEXT;
use crate::fs::{File, Stdin, Stdout};
use crate::mm::{MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE};
use crate::sync::UPSafeCell;
use crate::trap::{trap_handler, TrapContext};
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefMut;

/// Task status
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// Ready to run
    Ready,
    /// Running
    Running,
    /// Zombie (exited)
    Zombie,
}

/// Task control block structure
pub struct TaskControlBlock {
    /// immutable
    pub pid: PidHandle,
    /// kernel stack
    pub kernel_stack: KernelStack,
    /// mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

/// Task control block inner structure
pub struct TaskControlBlockInner {
    /// trap context physical page number
    pub trap_cx_ppn: PhysPageNum,
    /// application data only in application space
    pub base_size: usize,
    /// task context
    pub task_cx: TaskContext,
    /// task status
    pub task_status: TaskStatus,
    /// application address space
    pub memory_set: MemorySet,
    /// parent process
    pub parent: Option<Weak<TaskControlBlock>>,
    /// children processes
    pub children: Vec<Arc<TaskControlBlock>>,
    /// exit code
    pub exit_code: i32,
    /// file descriptor table
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
    /// heap bottom
    pub heap_bottom: usize,
    /// program break
    pub program_brk: usize,
    /// task priority for stride scheduling
    pub priority: isize,
    /// task stride for stride scheduling
    pub stride: usize,
}

impl TaskControlBlockInner {
    /// Get trap context
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    /// Get user token
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    /// Get status
    pub fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    /// Check if zombie
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
    /// Allocate a new file descriptor
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}

impl TaskControlBlock {
    /// Get the mutable reference of the inner TCB
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }

    /// Create a new task
    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // allocate a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = kstack_alloc();
        let kernel_stack_top = kernel_stack.get_top();
        // push a task context which goes to trap_return to the top of kernel stack
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                    heap_bottom: user_sp,
                    program_brk: user_sp,
                    priority: 16,  // Default priority
                    stride: 0,     // Initial stride
                })
            },
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }

    /// Load a new elf to replace the original application address space and start execution
    pub fn exec(&self, elf_data: &[u8]) {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        // **** access inner exclusively
        let mut inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.memory_set = memory_set;
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // initialize trap_cx
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
        // **** release inner automatically
    }

    /// Fork from parent to child
    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        // ---- access parent PCB exclusively
        let mut parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // allocate a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = kstack_alloc();
        let kernel_stack_top = kernel_stack.get_top();
        // copy fd table
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_inner.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: new_fd_table,
                    heap_bottom: parent_inner.heap_bottom,
                    program_brk: parent_inner.program_brk,
                    priority: parent_inner.priority,  // Inherit priority
                    stride: parent_inner.stride,      // Inherit stride
                })
            },
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** access children PCB exclusively
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        task_control_block
        // ---- release parent PCB automatically
        // **** release children PCB automatically
    }

    /// Get pid
    pub fn getpid(&self) -> usize {
        self.pid.0
    }

    /// Get user token
    pub fn get_user_token(&self) -> usize {
        let inner = self.inner_exclusive_access();
        inner.get_user_token()
    }

    /// Set priority
    pub fn set_priority(&self, prio: isize) {
        const BIG_STRIDE: usize = 0x7FFFFFFF;
        let mut inner = self.inner_exclusive_access();
        inner.priority = prio;
        inner.stride = BIG_STRIDE / prio.max(1) as usize;
    }

    /// Change program break for sbrk syscall
    pub fn change_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.inner_exclusive_access();
        let old_brk = inner.program_brk;
        let new_brk = inner.program_brk as isize + size as isize;
        if new_brk < inner.heap_bottom as isize {
            return None;
        }
        let old_program_brk = inner.program_brk;
        let result = inner.memory_set.sbrk(old_program_brk, new_brk as usize);
        if result {
            inner.program_brk = new_brk as usize;
            Some(old_brk)
        } else {
            None
        }
    }
}
