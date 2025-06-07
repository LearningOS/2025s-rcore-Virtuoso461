# Lab4 实验报告

## 实验概述

本次实验是rCore操作系统第六章的练习，主要涉及文件系统的硬链接功能实现。第六章在easy-fs文件系统的基础上，实现了硬链接相关的系统调用，包括创建硬链接、删除链接以及获取文件状态等功能。

## 实验内容

### 主要实现功能

1. **硬链接系统调用**
   - `sys_linkat`: 创建文件的硬链接
   - `sys_unlinkat`: 删除文件链接
   - `sys_fstat`: 获取文件状态信息

2. **文件系统扩展**
   - 支持多个目录项指向同一个inode
   - 实现链接计数管理
   - 支持文件的完全删除和回收

3. **错误处理机制**
   - 链接同名文件检测
   - 文件不存在错误处理
   - 文件描述符有效性检查

### 系统调用实现细节

#### sys_linkat 实现
```rust
pub fn sys_linkat(
    _olddirfd: i32,
    oldpath: *const u8,
    _newdirfd: i32, 
    newpath: *const u8,
    _flags: u32
) -> isize {
    // 获取路径字符串
    let token = current_user_token();
    let oldpath = translated_str(token, oldpath);
    let newpath = translated_str(token, newpath);
    
    // 检查是否为同名文件
    if oldpath == newpath {
        return -1;
    }
    
    // 创建硬链接
    if let Some(inode) = open_file(oldpath.as_str(), OpenFlags::RDONLY) {
        if let Some(old_inode) = inode.as_any().downcast_ref::<OSInode>() {
            if link(newpath.as_str(), old_inode.get_inode_id()) {
                0
            } else {
                -1
            }
        } else {
            -1
        }
    } else {
        -1
    }
}
```

#### sys_unlinkat 实现
```rust
pub fn sys_unlinkat(_dirfd: i32, path: *const u8, _flags: u32) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    
    if unlink(path.as_str()) {
        0
    } else {
        -1
    }
}
```

#### sys_fstat 实现
```rust
pub fn sys_fstat(fd: i32, st: *mut Stat) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    
    if fd as usize >= inner.fd_table.len() {
        return -1;
    }
    
    if let Some(file) = &inner.fd_table[fd as usize] {
        let stat = file.fstat();
        *translated_refmut(token, st) = stat;
        0
    } else {
        -1
    }
}
```

### 文件系统核心修改

#### Inode链接计数管理
```rust
pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect1: u32,
    pub indirect2: u32,
    pub type_: DiskInodeType,
    pub nlink: u32,  // 新增：硬链接计数
}
```

#### 硬链接创建逻辑
```rust
impl EasyFileSystem {
    pub fn link(&self, name: &str, inode_id: u32) -> bool {
        let mut root_inode = self.get_root_inode();
        
        // 检查文件是否已存在
        if root_inode.find(name).is_some() {
            return false;
        }
        
        // 增加链接计数
        let mut disk_inode = self.get_disk_inode(inode_id);
        disk_inode.nlink += 1;
        
        // 创建目录项
        root_inode.create(name, inode_id);
        true
    }
}
```

#### 文件删除逻辑
```rust
impl EasyFileSystem {
    pub fn unlink(&self, name: &str) -> bool {
        let mut root_inode = self.get_root_inode();
        
        if let Some(inode_id) = root_inode.find(name) {
            // 删除目录项
            root_inode.remove(name);
            
            // 减少链接计数
            let mut disk_inode = self.get_disk_inode(inode_id);
            disk_inode.nlink -= 1;
            
            // 如果链接计数为0，回收inode和数据块
            if disk_inode.nlink == 0 {
                self.dealloc_inode(inode_id);
                disk_inode.dealloc_data(&self.block_device);
            }
            
            true
        } else {
            false
        }
    }
}
```

### Stat结构体实现
```rust
#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    pub dev: u64,      // 设备号，固定为0
    pub ino: u64,      // inode编号
    pub mode: StatMode, // 文件类型
    pub nlink: u32,    // 硬链接数量
    pad: [u64; 7],     // 兼容性填充
}

bitflags! {
    pub struct StatMode: u32 {
        const NULL  = 0;
        const DIR   = 0o040000;   // 目录
        const FILE  = 0o100000;   // 普通文件
    }
}
```

## 实验结果

通过实现上述功能，成功完成了第六章的所有要求：
1. 硬链接创建和删除功能正常工作
2. 文件状态获取功能正确实现
3. 链接计数管理机制完善
4. 错误处理机制健全
5. 通过了所有测试用例，保持前向兼容性

## 问答作业

### root inode的作用及损坏影响

#### root inode的作用

在easy-fs文件系统中，root inode起着至关重要的作用：

1. **文件系统入口点**
   - root inode是整个文件系统的根目录
   - 所有文件和目录的访问都从root inode开始
   - 它是文件系统树形结构的根节点

2. **目录管理中心**
   - 存储根目录下所有文件和子目录的目录项
   - 管理文件名到inode编号的映射关系
   - 提供文件查找、创建、删除等基本操作

3. **文件系统元数据管理**
   - 维护根目录的大小、类型等元信息
   - 管理根目录的数据块分配
   - 控制根目录的访问权限

#### root inode损坏的影响

如果root inode中的内容损坏了，会发生以下严重后果：

1. **文件系统完全不可访问**
   - 无法找到任何文件或目录
   - 所有文件操作都会失败
   - 整个文件系统变得不可用

2. **数据丢失风险**
   - 即使数据块本身完好，也无法通过正常途径访问
   - 文件名到inode的映射关系丢失
   - 可能导致数据块泄漏（无法回收）

3. **系统启动失败**
   - 如果操作系统依赖该文件系统启动，可能导致系统无法启动
   - 关键系统文件无法访问

4. **恢复困难**
   - 需要专门的文件系统恢复工具
   - 可能需要重建整个目录结构
   - 数据恢复成功率不确定

## 荣誉准则

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

我在实现硬链接功能时参考了课程讲义和教材，在遇到文件系统操作问题时查阅了相关文档。

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

1. **rCore Tutorial Book Chapter 6** - 文件系统、硬链接实现（在 `easy-fs/src/` 相关文件中标注）
2. **UNIX文件系统设计** - 硬链接原理和实现（在 `easy-fs/src/vfs.rs` 中标注）
3. **Rust标准库文档** - 文件操作API设计（在 `os/src/syscall/fs.rs` 中标注）

我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按"-100"分计。

## 总结

本次实验成功实现了文件系统的硬链接功能，包括链接创建、删除和状态查询等核心功能。通过实现这些功能，深入理解了文件系统的内部结构和工作原理，特别是inode管理、目录项操作和链接计数机制。同时，通过完善的错误处理机制，提高了文件系统的健壮性和可靠性。
