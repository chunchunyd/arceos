use axconfig::{SMP, TASK_STACK_SIZE};
use axhal::mem::{virt_to_phys, VirtAddr};

#[link_section = ".bss.stack"]
static mut SECONDARY_BOOT_STACK: [[u8; TASK_STACK_SIZE]; SMP - 1] = [[0; TASK_STACK_SIZE]; SMP - 1];

extern "C" {
    fn _start_secondary();
}

pub fn start_secondary_cpus(primary_cpu_id: usize) {
    let entry = virt_to_phys(VirtAddr::from(_start_secondary as usize));
    let mut logic_cpu_id = 0;
    for i in 0..SMP {
        if i != primary_cpu_id {
            let stack_top = virt_to_phys(VirtAddr::from(unsafe {
                SECONDARY_BOOT_STACK[logic_cpu_id].as_ptr_range().end as usize
            }));

            debug!("starting CPU {}...", i);
            axhal::mp::start_secondary_cpu(i, entry, stack_top);
            logic_cpu_id += 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_main_secondary(cpu_id: usize) -> ! {
    info!("Secondary CPU {} started.", cpu_id);
    loop {
        axhal::arch::wait_for_irqs(); // TODO
    }
}
