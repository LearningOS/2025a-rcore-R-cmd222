# Lab1 Report

## 荣誉准则

在完成本次实验的过程（含此前学习的过程）中，我曾分别与以下各位就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

**交流对象说明**：
- ChatGPT：就 Rust 语言特性（如 `UPSafeCell`、`RefCell`、`lazy_static!`、`core::array::try_from_fn`、`alloc` crate 等）进行咨询，以及编译错误调试和系统调用实现逻辑的讨论
- Gemini：就 RISC-V 架构特性和系统调用设计模式进行交流

此外，我也参考了以下资料，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

**参考资料说明**：
- [Writing an OS in Rust](https://os.phil-opp.com/zh-CN)：参考了 Rust 操作系统开发的基础概念和实现模式
- [rCore-Tutorial-Code-2025S](https://learningos.cn/rCore-Tutorial-Code-2025S/ch4/os/index.html)：参考了 rCore 教程的代码结构和模块设计

我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按"-100"分计。

## 实现功能总结

本次实验实现了 rCore 第3章的 sys_trace 系统调用功能，主要包括：

**核心功能**：
- 实现了 sys_trace 的三种操作模式：内存读取（request=0）、内存写入（request=1）、系统调用次数查询（request=2）
- 支持按任务维度统计各类系统调用的调用次数，避免全局统计导致的计数混乱

**技术实现**：
- 将系统调用统计从全局单一计数器改为每个任务独立的计数器表（Vec<BTreeMap>）
- 在系统调用分发器中为 WRITE/EXIT/YIELD/GET_TIME 添加计数逻辑
- sys_trace 内部自增 TRACE 类型计数，确保"本次调用也计入统计"
- 移除了调试输出，避免污染用户程序输出和系统调用计数

**测试结果**：
所有第3章测试用例通过（7/7），包括时间获取、睡眠、写入和追踪功能测试。

## 问答题

### 1. 用户态程序错误行为测试

**使用的 SBI 版本**：RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0

**测试结果**：
- `ch2b_bad_address.rs`：尝试向地址 0x0 写入数据，触发 PageFault，内核输出 "PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it."
- `ch2b_bad_instructions.rs`：在用户态执行 `sret` 指令，触发 IllegalInstruction，内核输出 "IllegalInstruction in application, kernel killed it."
- `ch2b_bad_register.rs`：在用户态读取 `sstatus` 寄存器，触发 IllegalInstruction，内核输出 "IllegalInstruction in application, kernel killed it."

**分析**：正确进入用户态后，程序使用 S 态特权指令（如 `sret`、`csrr`）或访问 S 态寄存器（如 `sstatus`）会触发异常，被内核捕获并终止，这验证了特权级保护机制的有效性。

### 2. trap.S 汇编代码分析

**L40：刚进入 __restore 时，sp 代表了什么值？__restore 的两种使用情景？**

刚进入 `__restore` 时，`sp` 指向内核栈上已分配的 `TrapContext` 结构体的起始位置。`__restore` 有两种使用情景：
1. 从异常/中断处理完成后返回到用户态
2. 首次启动用户程序时从内核态切换到用户态

**L43-L48：特殊处理的寄存器及其意义**

这几行代码特殊处理了三个关键寄存器：
- `sstatus` (t0)：控制处理器的特权级状态，决定返回用户态时的特权级
- `sepc` (t1)：异常程序计数器，保存异常发生时的指令地址，用于异常返回
- `sscratch` (t2)：临时寄存器，用于保存用户栈指针，实现内核栈和用户栈的切换

**L50-L56：为何跳过了 x2 和 x4？**

- 跳过 `x2` (sp)：栈指针需要特殊处理，在 L60 通过 `csrrw` 指令与 `sscratch` 交换
- 跳过 `x4` (tp)：线程指针在应用程序中不使用，无需恢复

**L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？**

`csrrw sp, sscratch, sp` 执行后：
- `sp`：指向用户栈指针，准备返回用户态
- `sscratch`：指向内核栈指针，为下次异常处理做准备

**__restore 中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？**

状态切换发生在 `sret` 指令（L61）。`sret` 指令会：
1. 将 `sepc` 的值加载到 `pc`，跳转到用户程序
2. 将 `sstatus` 的 SPP 位清零，切换到用户态（U 态）
3. 恢复中断使能状态

**L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？**

`csrrw sp, sscratch, sp` 执行后：
- `sp`：指向内核栈，用于保存异常上下文
- `sscratch`：指向用户栈，保存用户程序的栈指针

**从 U 态进入 S 态是哪一条指令发生的？**

从用户态进入内核态不是通过特定指令，而是通过异常/中断机制自动触发的。当发生异常（如 PageFault、IllegalInstruction）或中断时，硬件会自动：
1. 将当前 `pc` 保存到 `sepc`
2. 将当前状态保存到 `sstatus`
3. 跳转到异常处理入口点（`stvec` 指向的地址）
4. 切换到内核态（S 态）
