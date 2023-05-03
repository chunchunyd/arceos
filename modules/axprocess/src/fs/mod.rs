pub mod file;
pub mod file_io;
pub mod stdio;


pub use file::new_fd;
pub use stdio::{Stdin, Stdout, Stderr};