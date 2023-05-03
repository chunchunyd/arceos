#![cfg_attr(not(test), no_std)]
pub mod time;

pub use axlog::{ax_print, ax_println, debug, error, info, trace, warn};

#[cfg(feature = "alloc")]
extern crate alloc;

extern crate axlog;

extern crate axruntime;

pub mod io;
pub mod rand;
pub mod sync;

pub mod syscall;
pub use syscall::*;

#[cfg(feature = "multitask")]
pub mod task;

#[cfg(feature = "net")]
pub mod net;

#[cfg(feature = "display")]
pub mod display;
