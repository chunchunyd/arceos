use alloc::{collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use axhal::arch::{write_page_table_root, TrapFrame};
pub const USER_STACK_SIZE: usize = 4096;
const KERNEL_STACK_SIZE: usize = 4096;
use crate::{
    current,
    mem::memory_set::{get_app_data, MemorySet},
    run_idle,
    run_queue::RUN_QUEUE,
    task::TaskInner,
    AxTaskRef, TaskId, syscall::clone_flags::CloneFlags,
};
use spinlock::SpinNoIrq;
const IDLE_TASK_STACK_SIZE: usize = 4096;
use riscv::asm;
pub(crate) static PID2PC: SpinNoIrq<BTreeMap<u64, Arc<Process>>> = SpinNoIrq::new(BTreeMap::new());
pub const KERNEL_PROCESS_ID: u64 = 1;
/// 进程的的数据结构
pub struct Process {
    /// 进程的pid和初始化的线程的tid是一样的
    pub pid: u64,
    pub inner: SpinNoIrq<ProcessInner>,
}

pub struct ProcessInner {
    /// 父进程的进程号
    pub parent: u64,
    /// 子进程
    pub children: Vec<Arc<Process>>,
    /// 子任务
    pub tasks: Vec<AxTaskRef>,
    /// 地址空间，由于存在地址空间共享，因此设计为Arc类型
    pub memory_set: Arc<SpinNoIrq<MemorySet>>,
    /// 进程状态
    pub is_zombie: bool,
    /// 退出状态码
    pub exit_code: i32,
}

impl ProcessInner {
    pub fn new(parent: u64, memory_set: Arc<SpinNoIrq<MemorySet>>) -> Self {
        Self {
            parent,
            children: Vec::new(),
            tasks: Vec::new(),
            memory_set,
            is_zombie: false,
            exit_code: 0,
        }
    }
}

impl Process {
    /// 初始化内核调度任务的进程
    pub fn new_kernel() -> AxTaskRef {
        // 内核进程的ID必定为1
        let kernel_process = Arc::new(Self {
            pid: TaskId::new().as_u64(),
            inner: SpinNoIrq::new(ProcessInner::new(0, Arc::new(SpinNoIrq::new(MemorySet::new_empty())))),
        });
        PID2PC
            .lock()
            .insert(kernel_process.pid, Arc::clone(&kernel_process));
        let new_task = TaskInner::new(
            || run_idle(),
            "idle",
            IDLE_TASK_STACK_SIZE,
            KERNEL_PROCESS_ID,
        );
        kernel_process
            .inner
            .lock()
            .tasks
            .push(Arc::clone(&new_task));
        // 不用为内核任务设立trap上下文
        new_task.set_leader(true);
        new_task
    }

    /// 根据name新建一个进程
    /// 此时应当是初始化的主进程，所以其父亲应当是内核进程
    pub fn new(name: &'static str) -> AxTaskRef {
        // 接下来是加载自己的内容
        // let mut page_table = copy_from_kernel_memory();
        // let (entry, user_stack_bottom) = load_from_elf(&mut page_table, get_app_data(name));
        let mut memory_set = MemorySet::new_from_kernel();
        let (entry, user_stack_bottom) = MemorySet::from_elf(&mut memory_set, get_app_data(name));
        // 以这种方式建立的线程，不通过某一个具体的函数开始，而是通过地址来运行函数，所以entry不会被用到
        let new_process = Arc::new(Self {
            pid: TaskId::new().as_u64(),
            inner: SpinNoIrq::new(ProcessInner::new(KERNEL_PROCESS_ID, Arc::new(SpinNoIrq::new(memory_set)))),
        });
        // 记录该进程，防止被回收
        PID2PC
            .lock()
            .insert(new_process.pid, Arc::clone(&new_process));
        // 创立一个新的线程，初始化时进入
        let new_task = TaskInner::new(|| {}, name, KERNEL_STACK_SIZE, new_process.pid);
        new_task.set_leader(true);
        // 初始化线程的trap上下文
        let new_trap_frame = TrapFrame::app_init_context(entry, user_stack_bottom + USER_STACK_SIZE);
        new_task.set_trap_context(new_trap_frame) ;
        // 设立父子关系
        let mut inner = new_process.inner.lock();
        inner.tasks.push(Arc::clone(&new_task));
        drop(inner);
        new_task.set_trap_in_kernel_stack();
        new_task
        // let kernel_sp = new_task.get_kernel_stack_top();
    }
    /// 将当前进程替换为指定的用户程序
    /// args为传入的参数
    pub fn exec(&self, elf_data: &[u8], args: Vec<String>) {
        // 首先要处理原先进程的资源
        // 处理分配的页帧
        let mut inner = self.inner.lock();
        inner.memory_set.lock().unmap_user_areas();
        // inner.memory_set.areas.clear();
        // let mut new_memory_set = MemorySet::new_from_kernel();
        // 之后加入额外的东西之后再处理其他的包括信号等因素
        // 不是直接删除原有地址空间，否则构建成本较高。
        // 再考虑手动结束其他所有的task
        unsafe {
            asm::sfence_vma_all();
        }
        let curr = current();
        let _ = inner
            .tasks
            .drain_filter(|task: &mut Arc<scheduler::FifoTask<TaskInner>>| task.id() != curr.id())
            .map(|task| RUN_QUEUE.lock().remove_task(&task));
        // 当前任务被设置为主线程
        curr.set_leader(true);
        assert!(inner.tasks.len() == 1);
        let (entry, user_stack_bottom) = MemorySet::from_elf(&mut inner.memory_set.lock(), elf_data);
        // inner.memory_set = new_memory_set;
        // 切换了地址空间， 需要切换token
        let page_table_token = if self.pid == KERNEL_PROCESS_ID {
            0
        } else {
            inner.memory_set.lock().page_table_token()
        };
        if page_table_token != 0 {
            // axhal::arch::write_page_table_root(page_table_token.into());
            unsafe { write_page_table_root(page_table_token.into()) };
        }
        drop(inner);
        let mut user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        user_stack_top -= (args.len() + 1) * core::mem::size_of::<usize>();
        let argv_base = user_stack_top;
        let mut argv: Vec<_> = (0..=args.len())
            .map(|arg| (argv_base + arg * core::mem::size_of::<usize>()) as *mut usize)
            .collect();
        unsafe {
            *argv[args.len()] = 0;
        }
        for i in 0..args.len() {
            user_stack_top -= args[i].len() + 1;
            unsafe {
                *argv[i] = user_stack_top;
            }
            let mut p = user_stack_top;
            for c in args[i].as_bytes() {
                unsafe {
                    *(p as *mut u8) = *c;
                }
                p += 1;
            }
            unsafe {
                *(p as *mut u8) = 0;
            }
        }
        // 对齐到4K
        user_stack_top -= user_stack_top % core::mem::size_of::<usize>();
        // user_stack_top = user_stack_top / PAGE_SIZE_4K * PAGE_SIZE_4K;
        let new_trap_frame = TrapFrame::app_init_context(entry, user_stack_top);
        curr.set_trap_context(new_trap_frame);
        let frame_address = curr.trap_frame.get() as usize;
        unsafe {
            // curr.trap_frame.get_mut().regs.a0 = args.len();
            // curr.trap_frame.get_mut().regs.a1 = argv_base;
            *((frame_address + 9 * core::mem::size_of::<usize>()) as *mut usize) = args.len();
            *((frame_address + 10 * core::mem::size_of::<usize>()) as *mut usize) = argv_base;
        }
        curr.set_trap_in_kernel_stack();
    }
    /// 实现简易的clone系统调用
    /// 返回值为新产生的任务的id
    pub fn clone(&self, flags: CloneFlags, stack: Option<usize>, ptid: usize, tls: usize, ctid: usize) -> u64 {
        let mut inner = self.inner.lock();
        // 是否共享虚拟地址空间
        let new_memory_set = if flags.contains(CloneFlags::CLONE_VM) {
            // 若是则直接共享指针即可
            Arc::clone(&inner.memory_set)
        } else {
            // 否则复制地址空间
            Arc::new(SpinNoIrq::new(MemorySet::new_from_task(&(inner.memory_set.lock())))) 
        };

        // 在生成新的进程前，需要决定其所属进程是谁
        let process_id = if flags.contains(CloneFlags::CLONE_THREAD) {
            // 当前clone生成的是线程，那么以self作为进程
            self.pid
        } else {
            // 新建一个进程，并且设计进程之间的父子关系
            TaskId::new().as_u64()
        };
        // 决定父进程是谁
        let parent_id = if flags.contains(CloneFlags::CLONE_PARENT) {
            // 创建兄弟关系，此时以self的父进程作为自己的父进程
            // 理论上不应该创建内核进程的兄弟进程，所以可以直接unwrap
            inner.parent
        }
        else {
            // 创建父子关系，此时以self作为父进程
            self.pid
        };
        // let new_process = 
        let new_task = TaskInner::new(|| {}, "", KERNEL_STACK_SIZE, process_id);
        // 决定是创建线程还是进程
        if flags.contains(CloneFlags::CLONE_THREAD) {
            // 若创建的是进程，那么不用新建进程
            inner.tasks.push(Arc::clone(&new_task));
        }
        else {
            // 若创建的是进程，那么需要新建进程
            let new_process = Arc::new(Self{pid: process_id, inner: SpinNoIrq::new(ProcessInner::new(parent_id, new_memory_set))});
            // 记录该进程，防止被回收
            PID2PC
            .lock()
            .insert(process_id, Arc::clone(&new_process));
            new_process.inner.lock().tasks.push(Arc::clone(&new_task));
        };
        drop(inner);
        if !flags.contains(CloneFlags::CLONE_THREAD) {
            new_task.set_leader(true);
        }
        let curr = current();
        let mut trap_frame = unsafe {
            *(curr.get_first_trap_frame())
        };
        drop(curr);
        // 新开的进程/线程返回值为0
        trap_frame.regs.a0 = 0;
        if flags.contains(CloneFlags::CLONE_SETTLS) {
            trap_frame.regs.tp = tls;
        }
        // 设置用户栈
        // 若给定了用户栈，则使用给定的用户栈
        // 若没有给定用户栈，则使用当前用户栈
        // 没有给定用户栈的时候，只能是共享了地址空间，且原先调用clone的有用户栈，此时已经在之前的trap clone时复制了
        if let Some(stack) = stack {
            trap_frame.regs.sp = stack;
            info!("New user stack: sepc:{:X}, stack:{:X}", trap_frame.sepc, trap_frame.regs.sp);
        }
        new_task.set_trap_context(trap_frame);
        new_task.set_trap_in_kernel_stack();
        new_task.id().as_u64()
    }
}

/// 初始化进程的trap上下文
#[no_mangle]
// #[cfg(feature = "user")]
pub fn first_into_user(kernel_sp: usize, frame_base: usize) -> ! {
    let trap_frame_size = core::mem::size_of::<TrapFrame>();
    let kernel_base = kernel_sp - trap_frame_size;
    unsafe {
        core::arch::asm!(
            r"
            mv      sp, {frame_base}
            LDR     gp, sp, 2                   // load user gp and tp
            LDR     t0, sp, 3
            mv      t1, {kernel_base}
            STR     tp, t1, 3                   // save supervisor tp，注意是存储到内核栈上而不是sp中
            mv      tp, t0                      // tp：线程指针
            csrw    sscratch, {kernel_sp}       // put supervisor sp to scratch
            LDR     t0, sp, 31
            LDR     t1, sp, 32
            csrw    sepc, t0
            csrw    sstatus, t1
            POP_GENERAL_REGS
            LDR     sp, sp, 1
            sret
        ",
            frame_base = in(reg) frame_base,
            kernel_sp = in(reg) kernel_sp,
            kernel_base = in(reg) kernel_base,
        );
    };
    core::panic!("already in user mode!")
}

// #[cfg(not(feature = "user"))]
// pub fn first_into_user(_kernel_sp: usize, _frame_base: usize) -> ! {
//     extern "Rust" {
//         fn main();
//     }

//     unsafe { main() };
//     crate::exit(0)
// }
