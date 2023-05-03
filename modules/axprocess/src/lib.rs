#![cfg_attr(not(test), no_std)]
#![feature(drain_filter)]
extern crate alloc;

pub mod process;
pub mod mem;
pub mod signal;
pub mod fs;

use alloc::sync::Arc;
use axtask::current;
use process::{Process, PID2PC};


pub fn current_process() -> Arc<Process> {
    let cur_task = current();
    let process_id = cur_task.get_process_id();
    drop(cur_task);

    let pid2pc_inner = PID2PC.lock();
    Arc::clone(&pid2pc_inner.get(&process_id).unwrap())
}

