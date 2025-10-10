//! Process management syscalls
#![allow(unused_imports)]
use core::array::try_from_fn;

use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// tracing / stats syscall
// request semantics:
// 0 -> read memory at address (arg1)
// 1 -> write memory at address (arg1) with value (arg2)
// 2 -> return syscall count of given syscall type (arg1), including this call
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace request={} id={}", trace_request, id);
    
    // 先统计 SYSCALL_TRACE 的调用次数
    
    match trace_request {
        0 => {
            // read memory
            let addr = id as *const u8;
            unsafe { core::ptr::read_volatile(addr) as isize }
        }
        1 => {
            // write memory
            let addr = id as *mut u8;
            unsafe { core::ptr::write_volatile(addr, data as u8); }
            0
        }
        2 => {
            // syscall type count (including this call)
            crate::task::incr_syscall_type(410); // SYSCALL_TRACE = 410
            let count = crate::task::get_syscall_type_count(id);
            count as isize
        }
        _ => -1,
    }
}
