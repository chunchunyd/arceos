#![no_std]
#![no_main]

#[no_mangle]
fn main() {
    libax::syscall::sys_write(1, b"Hello, ooo!\n");
}
