use riscv::register::satp;

use axconfig::{PHYS_VIRT_OFFSET, TASK_STACK_SIZE};

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; TASK_STACK_SIZE] = [0; TASK_STACK_SIZE];

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT_SV39: [u64; 512] = [0; 512];

unsafe fn init_boot_page_table() {
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV39[2] = (0x80000 << 10) | 0xef;
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    BOOT_PT_SV39[0x102] = (0x80000 << 10) | 0xef;
}

pub unsafe fn sfence_vma() {
    riscv::asm::sfence_vma_all();
}

unsafe fn init_mmu() {
    let page_table_root = BOOT_PT_SV39.as_ptr() as usize;
    // 启用页表，并且规定使用唯一的根页表
    satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
    riscv::register::sstatus::set_sum();
}

#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start() -> ! {
    extern "Rust" {
        fn rust_main();
    }
    // PC = 0x8020_0000
    // a0 = hartid
    // a1 = dtb
    core::arch::asm!("
        mv      s0, a0                  // save hartid
        mv      s1, a1                  // save DTB pointer
        la      sp, {boot_stack}
        li      t0, {boot_stack_size}
        add     sp, sp, t0              // setup boot stack

        call    {init_boot_page_table}
        call    {init_mmu}              // setup boot page table and enabel MMU

        li      s2, {phys_virt_offset}  // fix up virtual high address
        add     sp, sp, s2

        mv      a0, s0
        mv      a1, s1
        la      a2, {platform_init}
        add     a2, a2, s2
        jalr    a2                      // call platform_init(hartid, dtb)

        mv      a0, s0
        mv      a1, s1
        la      a2, {rust_main}
        add     a2, a2, s2
        jalr    a2                      // call rust_main(hartid, dtb)
        j       .",
        phys_virt_offset = const PHYS_VIRT_OFFSET,
        boot_stack_size = const TASK_STACK_SIZE,
        boot_stack = sym BOOT_STACK,
        init_boot_page_table = sym init_boot_page_table,
        init_mmu = sym init_mmu,
        platform_init = sym super::platform_init,
        rust_main = sym rust_main,
        options(noreturn),
    )
}

#[cfg(feature = "smp")]
#[naked]
#[no_mangle]
#[link_section = ".text.boot"]
unsafe extern "C" fn _start_secondary() -> ! {
    extern "Rust" {
        fn rust_main_secondary();
    }
    // a0 = hartid
    // a1 = SP
    core::arch::asm!("
        mv      s0, a0                  // save hartid
        mv      sp, a1                  // set SP

        call    {init_mmu}              // setup boot page table and enabel MMU

        li      s1, {phys_virt_offset}  // fix up virtual high address
        add     a1, a1, s1
        add     sp, sp, s1

        mv      a0, s0
        la      a1, {platform_init_secondary}
        add     a1, a1, s1
        jalr    a1                      // call platform_init_secondary(hartid)

        mv      a0, s0
        la      a1, {rust_main_secondary}
        add     a1, a1, s1
        jalr    a1                      // call rust_main_secondary(hartid)
        j       .",
        phys_virt_offset = const PHYS_VIRT_OFFSET,
        init_mmu = sym init_mmu,
        platform_init_secondary = sym super::platform_init_secondary,
        rust_main_secondary = sym rust_main_secondary,
        options(noreturn),
    )
}
