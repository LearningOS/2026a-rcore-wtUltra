//! Additional user-mode regression, with Codex assistance, based on the
//! official chapter6 hard-link specification and open-file lifetime semantics.
#![no_std]
#![no_main]
extern crate user_lib;
use user_lib::{
    close, fstat, link, open, println, read, syscall, unlink, write, OpenFlags, Stat, SYSCALL_FSTAT,
};

#[no_mangle]
pub fn main() -> i32 {
    let fd = open("lifecycle_old\0", OpenFlags::CREATE | OpenFlags::RDWR) as usize;
    let mut stat = Stat::new();
    assert_eq!(fstat(fd, &mut stat), 0);
    let original_id = stat.ino;
    assert_eq!(write(fd, b"alive after unlink"), 18);
    assert_eq!(close(fd), 0);
    let fd = open("lifecycle_old\0", OpenFlags::RDONLY) as usize;
    assert_eq!(link("lifecycle_old\0", "lifecycle_alias\0"), 0);
    assert_eq!(unlink("lifecycle_old\0"), 0);
    assert_eq!(unlink("lifecycle_alias\0"), 0);
    assert_eq!(fstat(fd, &mut stat), 0);
    assert_eq!(stat.ino, original_id);
    assert_eq!(stat.nlink, 0);
    let mut bytes = [0u8; 18];
    assert_eq!(read(fd, &mut bytes), 18);
    assert_eq!(&bytes, b"alive after unlink");
    assert_eq!(open("lifecycle_old\0", OpenFlags::RDONLY), -1);
    assert_eq!(syscall(SYSCALL_FSTAT, [fd, usize::MAX, 0]), -1);
    let fresh = open("lifecycle_new\0", OpenFlags::CREATE | OpenFlags::RDWR) as usize;
    assert_eq!(fstat(fresh, &mut stat), 0);
    assert_ne!(stat.ino, original_id);
    assert_eq!(close(fd), 0);
    assert_eq!(close(fd), -1);
    let recycled = open("lifecycle_recycled\0", OpenFlags::CREATE | OpenFlags::RDWR) as usize;
    assert_eq!(fstat(recycled, &mut stat), 0);
    assert_eq!(stat.ino, original_id);
    assert_eq!(close(recycled), 0);
    assert_eq!(unlink("lifecycle_recycled\0"), 0);
    assert_eq!(close(fresh), 0);
    assert_eq!(unlink("lifecycle_new\0"), 0);
    println!("Local unlink lifecycle regression OK!");
    0
}
