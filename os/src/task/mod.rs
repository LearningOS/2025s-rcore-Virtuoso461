//! Task management implementation
//!
//! 参考资料：rCore Tutorial Book Chapter 3 - 任务管理和调度机制

mod context;
mod switch;
mod task;


use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::vec::Vec;
use lazy_static::*;
pub use switch::__switch;
pub use context::TaskContext;

use task::{TaskControlBlock, TaskStatus};

/// The task manager, where all the tasks are managed.
///
/// Functions implemented:
/// - `run_first_task()`: start the first task
/// - `mark_current_suspended()`: mark current task as suspended
/// - `mark_current_exited()`: mark current task as exited
/// - `run_next_task()`: run the next task
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// The task manager inner in 'UPSafeCell'
struct TaskManagerInner {
    /// task list
    tasks: Vec<TaskControlBlock>,
    /// id of current `Running` task
    current_task: usize,
}

lazy_static! {
    /// a `TaskManager` instance through lazy_static!
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock {
                task_cx: TaskContext::goto_restore(init_app_cx(i)),
                task_status: TaskStatus::Ready,
                syscall_counts: alloc::collections::BTreeMap::new(),
                trace_data: alloc::collections::BTreeMap::new(),
            });
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut _, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.current_task
    }

    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        drop(inner);
        let trap_cx_ptr = init_app_cx(current);
        unsafe { &mut *(trap_cx_ptr as *mut TrapContext) }
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }

    /// Increment syscall count for current task
    fn increment_syscall_count(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let count = inner.tasks[current].syscall_counts.entry(syscall_id).or_insert(0);
        *count += 1;
    }

    /// Get syscall count for current task
    fn get_syscall_count(&self, syscall_id: usize) -> usize {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        *inner.tasks[current].syscall_counts.get(&syscall_id).unwrap_or(&0)
    }

    /// Write trace data for current task
    fn trace_write(&self, id: usize, data: u8) {
        // 首先存储到 trace_data 中
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].trace_data.insert(id, data);
        drop(inner);

        // 同时尝试写入实际的内存地址
        unsafe {
            let ptr = id as *mut u8;
            // 扩大地址范围检查，包括用户程序地址空间
            if (id >= 0x80000000 && id < 0x88000000) || (id >= 0x10000 && id < 0x80000000) {
                ptr.write_volatile(data);
            }
        }
    }

    /// Read trace data for current task
    fn trace_read(&self, id: usize) -> Option<u8> {
        // 首先尝试从 trace_data 中读取
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        if let Some(data) = inner.tasks[current].trace_data.get(&id).copied() {
            drop(inner);
            return Some(data);
        }
        drop(inner);

        // 如果 trace_data 中没有，尝试直接从内存地址读取
        unsafe {
            let ptr = id as *const u8;
            // 扩大地址范围检查，包括用户程序地址空间
            if (id >= 0x80000000 && id < 0x88000000) || (id >= 0x10000 && id < 0x80000000) {
                Some(ptr.read_volatile())
            } else {
                None
            }
        }
    }
}

/// Run the first task in task list.
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// Switch current `Running` task to the task we have found,
/// or there is no `Ready` task and we can exit with all applications completed
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// Change the status of current `Running` task into `Ready`.
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// Change the status of current `Running` task into `Exited`.
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts.
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}

/// Increment syscall count for current task
pub fn increment_syscall_count(syscall_id: usize) {
    TASK_MANAGER.increment_syscall_count(syscall_id);
}

/// Get syscall count for current task
pub fn get_syscall_count(syscall_id: usize) -> usize {
    TASK_MANAGER.get_syscall_count(syscall_id)
}

/// Write trace data for current task
pub fn trace_write(id: usize, data: u8) {
    TASK_MANAGER.trace_write(id, data);
}

/// Read trace data for current task
pub fn trace_read(id: usize) -> Option<u8> {
    TASK_MANAGER.trace_read(id)
}
