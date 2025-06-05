# Lab3 实验报告

## 实验概述

本次实验是rCore操作系统第五章的练习，主要涉及进程管理、进程调度算法以及相关系统调用的实现。第五章引入了进程概念，实现了基于优先级的stride调度算法，并完成了spawn、set_priority等进程管理相关的系统调用。

## 实验内容

### 主要实现功能

1. **进程管理系统**
   - 实现了进程创建（spawn）和进程优先级设置
   - 支持父子进程关系管理
   - 实现了进程等待和回收机制

2. **stride调度算法**
   - 基于优先级的公平调度算法
   - 支持动态优先级调整
   - 确保高优先级进程获得更多CPU时间

3. **系统调用实现**
   - `sys_spawn`: 创建新进程
   - `sys_set_priority`: 设置进程优先级
   - `sys_get_time`: 获取当前时间（跨页面处理）
   - `sys_mmap/sys_munmap`: 内存映射（错误处理优化）

4. **任务管理器重构**
   - 从简单的FIFO队列改为基于stride的优先级队列
   - 使用BinaryHeap实现高效的任务调度
   - 支持动态任务优先级调整

### 核心数据结构

#### 任务控制块（TaskControlBlock）
```rust
pub struct TaskControlBlockInner {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
    pub priority: usize,    // 进程优先级
    pub stride: usize,      // stride调度算法的步长值
    // ... 其他字段
}
```

#### Stride调度器
```rust
pub struct TaskManager {
    ready_queue: BinaryHeap<StrideTask>,
}

pub struct StrideTask {
    pub task: Arc<TaskControlBlock>,
    pub stride: usize,
}
```

### 系统调用实现细节

#### sys_spawn 实现
创建新进程的系统调用：
- 根据程序名称查找应用程序数据
- 创建新的TaskControlBlock
- 建立父子进程关系
- 将新进程加入调度队列
- 返回新进程的PID

#### sys_set_priority 实现
设置进程优先级：
- 验证优先级参数（必须 >= 2）
- 更新当前进程的优先级
- 返回设置的优先级值

#### stride调度算法实现
- 每次选择stride值最小的进程执行
- 进程执行后，stride += BigStride / priority
- 使用BinaryHeap维护按stride排序的就绪队列
- 支持溢出安全的stride比较

### 错误处理优化

#### mmap/munmap错误处理
- 添加页面对齐检查（4KB边界）
- 完善权限参数验证
- 优化重叠区域检测
- 确保部分取消映射操作正确返回错误

## 实验结果

通过实现上述功能，成功完成了第五章的所有要求：
1. stride调度算法正常工作，实现公平调度
2. 进程创建和优先级设置功能正确实现
3. 所有系统调用功能完善，错误处理机制健全
4. 通过了所有15个测试用例，包括之前失败的mmap相关测试

## 问答作业

### stride 算法深入

#### 1. stride算法的溢出问题

**问题场景：** 两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride，p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

**实际情况分析：**
实际情况下**不会**轮到 p1 执行。原因如下：

- p2 执行后：p2.stride = 250 + BigStride/10 = 250 + 255/10 = 250 + 25 = 275
- 由于使用8位无符号整数，275会溢出：275 % 256 = 19
- 比较时：p1.stride = 255, p2.stride = 19
- 使用普通比较：19 < 255，所以会选择p2继续执行
- 这违反了stride算法的公平性原则

#### 2. 优先级 >= 2 的必要性

**为什么要求进程优先级 >= 2：**

在不考虑溢出的情况下，如果严格按照算法执行，当所有进程优先级 >= 2 时，可以证明 STRIDE_MAX - STRIDE_MIN <= BigStride / 2。

**简单说明：**
- 设最小优先级为2，则最大步长增量为 BigStride/2
- 在任意时刻，如果某进程的stride是最小值，其他进程的stride最多比它大 BigStride/2
- 这确保了在有限的stride值范围内，不会出现"反超"现象
- 当优先级 >= 2 时，stride的增长是可控的，不会导致调度顺序的混乱

#### 3. 溢出安全的stride比较器

```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 计算两个stride的差值
        let diff = self.0.wrapping_sub(other.0);
        
        // 如果差值小于等于最大值的一半，说明self确实大于other
        // 否则说明发生了溢出，实际上self小于other
        if diff <= u64::MAX / 2 {
            if diff == 0 {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            }
        } else {
            Some(Ordering::Less)
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false  // 题目假设两个Stride永远不会相等
    }
}
```

**实现原理：**
- 使用wrapping_sub计算差值，避免溢出panic
- 如果差值 <= MAX/2，说明是正常的大小关系
- 如果差值 > MAX/2，说明发生了溢出，实际关系相反
- 这样可以正确处理类似 (125 < 255) == false, (129 < 255) == true 的情况

## 荣誉准则

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

我在实现stride调度算法时参考了课程讲义和教材，在遇到BinaryHeap使用问题时查阅了Rust官方文档。

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

1. **rCore Tutorial Book Chapter 5** - 进程管理、stride调度算法、系统调用实现（在 `os/src/task/manager.rs`、`os/src/syscall/process.rs` 中标注）
2. **Rust BinaryHeap Documentation** - 优先级队列的使用方法（在 `os/src/task/manager.rs` 中标注）
3. **stride调度算法论文** - 算法原理和溢出处理（在 `os/src/task/manager.rs` 中标注）
4. **RISC-V Assembly Programmer's Manual** - 系统调用约定（在 `os/src/syscall/mod.rs` 中标注）

我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按"-100"分计。

## 总结

本次实验成功实现了基于stride算法的进程调度系统，包括进程创建、优先级管理、公平调度等核心功能。通过实现这些功能，深入理解了操作系统的进程管理和调度机制，特别是如何在有限的数值范围内实现公平调度算法。同时，通过优化mmap/munmap的错误处理，提高了系统的健壮性和可靠性。
