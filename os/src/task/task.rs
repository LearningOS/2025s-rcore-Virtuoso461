//! Types related to task management

use super::TaskContext;
use alloc::collections::BTreeMap;

/// The task control block (TCB) of a task.
#[derive(Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// System call counts for tracing
    pub syscall_counts: BTreeMap<usize, usize>,
    /// Memory trace data
    pub trace_data: BTreeMap<usize, u8>,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
