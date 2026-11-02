#![no_std]
#![no_main]
extern crate user_lib;
use user_lib::{mmap, munmap, println, syscall, trace_read, SYSCALL_GETTIMEOFDAY};

#[no_mangle]
pub fn main() -> i32 {
    const PAGE: usize = 4096;
    const BASE: usize = 0x2000_0000;
    assert_eq!(mmap(BASE, PAGE * 4, 3), 0);
    for page in 0..4 {
        unsafe { core::ptr::write_volatile((BASE + PAGE * page) as *mut u8, page as u8 + 11) };
    }
    let time = BASE + PAGE - 8;
    assert_eq!(syscall(SYSCALL_GETTIMEOFDAY, [time, 0, 0]), 0);
    assert!(unsafe { core::ptr::read_volatile((time + 8) as *const usize) } < 1_000_000);
    assert_eq!(munmap(BASE + PAGE, PAGE * 2), 0);
    assert_eq!(trace_read((BASE + PAGE) as *const u8), None);
    assert_eq!(trace_read(BASE as *const u8), Some(11));
    assert_eq!(trace_read((BASE + PAGE * 3) as *const u8), Some(14));
    unsafe { core::ptr::write_volatile(time as *mut usize, 0xdead_beef) };
    assert_eq!(syscall(SYSCALL_GETTIMEOFDAY, [time, 0, 0]), -1);
    assert_eq!(
        unsafe { core::ptr::read_volatile(time as *const usize) },
        0xdead_beef
    );
    assert_eq!(munmap(BASE, PAGE * 2), -1);
    assert_eq!(trace_read(BASE as *const u8), Some(11));
    assert_eq!(mmap(BASE + PAGE, PAGE * 2, 3), 0);
    assert_eq!(trace_read((BASE + PAGE) as *const u8), Some(0));
    assert_eq!(munmap(BASE, PAGE * 4), 0);
    assert_eq!(trace_read(BASE as *const u8), None);
    assert_eq!(mmap(BASE, usize::MAX, 3), -1);
    assert_eq!(munmap(BASE, usize::MAX), -1);
    assert_eq!(mmap(BASE, PAGE, 3), 0);
    assert_eq!(trace_read((BASE + (1usize << 39)) as *const u8), None);
    assert_eq!(syscall(SYSCALL_GETTIMEOFDAY, [usize::MAX - 7, 0, 0]), -1);
    assert_eq!(munmap(BASE, PAGE), 0);
    println!("Local memory boundary regression OK!");
    0
}
