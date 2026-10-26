// Implementation assistance: Codex AI; disclosed in reports.
// Specification: LearningOS/rCore-Tutorial-Guide source/chapter4/7exercise.rst.
//! Process management syscalls
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// Copy the complete timeval through checked user mappings, even across pages.
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = crate::timer::get_time_us();
    let time = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let bytes = unsafe {
        core::slice::from_raw_parts(
            &time as *const TimeVal as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    };
    if crate::mm::copy_to_user(crate::task::current_user_token(), ts as usize, bytes) {
        0
    } else {
        -1
    }
}

pub fn sys_trace(request: usize, id: usize, data: usize) -> isize {
    let token = crate::task::current_user_token();
    match request {
        0 => crate::mm::checked_user_buffer(token, id, 1, false)
            .map_or(-1, |buffers| buffers[0][0] as isize),
        1 => {
            if crate::mm::copy_to_user(token, id, &[data as u8]) {
                0
            } else {
                -1
            }
        }
        2 => crate::task::syscall_count(id) as isize,
        _ => -1,
    }
}

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    crate::task::mmap(start, len, prot)
}
pub fn sys_munmap(start: usize, len: usize) -> isize {
    crate::task::munmap(start, len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
