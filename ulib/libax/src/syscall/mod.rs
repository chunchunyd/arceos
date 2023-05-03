pub mod fs;
pub mod task;
pub mod syscall_id;

pub use fs::*;
pub use task::*;

use core::arch::asm;
use syscall_id::*;

pub fn syscall(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize = -1;
    #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
    unsafe {
        asm!(
        "ecall",
        inlateout("x10") args[0] => ret,
        in("x11") args[1],
        in("x12") args[2],
        in("x13") args[3],
        in("x14") args[4],
        in("x15") args[5],
        in("x17") id
        );
    }
    ret
}

pub fn sys_exit(exit_code: i32) -> ! {
    syscall(SYS_EXIT, [exit_code as usize, 0, 0, 0, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(
        SYS_WRITE,
        [fd, buffer.as_ptr() as usize, buffer.len(), 0, 0, 0],
    )
}
