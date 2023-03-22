use axconfig::{SMP, TASK_STACK_SIZE};
use axhal::{
    arch::wait_for_irqs,
    lcpu::{lcpu_start, lcpu_started},
    mem::{virt_to_phys, VirtAddr},
};

#[link_section = ".bss.stack"]
static mut SECONDARY_BOOT_STACK: [[u8; TASK_STACK_SIZE]; SMP - 1] = [[0; TASK_STACK_SIZE]; SMP - 1];

extern "C" {
    fn _start_secondary();
}

pub fn start_secondary_cpus(primary_cpu_id: usize) {
    lcpu_started();
    let entry = virt_to_phys(VirtAddr::from(_start_secondary as usize));
    let mut logic_cpu_id = 0;
    for i in 0..SMP {
        if i != primary_cpu_id {
            let stack_top = virt_to_phys(VirtAddr::from(unsafe {
                SECONDARY_BOOT_STACK[logic_cpu_id].as_ptr_range().end as usize
            }));

            debug!("starting CPU {}...", i);
            // todo: args should be address of args
            lcpu_start(i, entry, stack_top);
            logic_cpu_id += 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_main_secondary(cpu_id: usize) -> ! {
    info!("Secondary CPU {} started.", cpu_id);

    #[cfg(feature = "paging")]
    init_paging_secondary();

    info!("Secondary CPU {} init OK.", cpu_id);
    super::INITED_CPUS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);

    while !super::is_init_ok() {
        core::hint::spin_loop();
    }

    axhal::arch::enable_irqs();
    lcpu_started();
    loop {
        wait_for_irqs(); // TODO
    }
}

#[cfg(feature = "paging")]
fn init_paging_secondary() {}
