#![cfg_attr(not(test), no_std)]
use flags::WaitFlags;
use log::info;
use task::{syscall_clone, syscall_getpid, syscall_getppid, syscall_wait4};

use self::{
    fs::*,
    task::{syscall_exec, syscall_exit},
};
extern crate axlog;
extern crate log;

extern crate alloc;
mod flags;
mod fs;
mod task;
mod syscall_id;

pub use syscall_id::*;

// pub const SYSCALL_WRITE: usize = 64;
// pub const SYSCALL_EXIT: usize = 93;
// pub const SYSCALL_CLONE: usize = 220;
// pub const SYSCALL_EXEC: usize = 221;

#[no_mangle]
// #[cfg(feature = "user")]
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    match syscall_id {
        SYS_WRITE => syscall_write(args[0], args[1] as *const u8, args[2]),
        SYS_EXIT => syscall_exit(args[0] as i32),
        SYS_EXECVE => syscall_exec(args[0] as *const u8, args[1] as *const usize),
        SYS_CLONE => syscall_clone(args[0], args[1], args[2], args[3], args[4]),
        SYS_GETPID => syscall_getpid(),
        SYS_GETPPID => syscall_getppid(),
        SYS_WAIT4 => syscall_wait4(
            args[0] as isize,
            args[1] as *mut i32,
            WaitFlags::from_bits(args[2] as u32).unwrap(),
        ),
        SYS_READ =>syscall_read(args[0], args[1] as *mut u8, args[2]),
        SYS_OPENAT =>syscall_open(args[0], args[1] as *const u8, args[2] as u8, args[3] as u8),    // args[0] is fd, args[1] is filename, args[2] is flags, args[3] is mode
        SYS_CLOSE =>syscall_close(args[0]), // args[0] is fd
        _ => {
            panic!("Invalid Syscall Id: {}!", syscall_id);
        }
    }
}
