use axalloc::GlobalPage;
use axhal::paging::MappingFlags;
use memory_addr::VirtAddr;
/// 地址段实现
/// 仅会给进程使用，内核不会改动其原有代码。
pub struct MapArea {
    /// global page本身就是多个页面的，且存储了起始地址
    pub start_va: VirtAddr,
    pub pages: GlobalPage,
    pub flags: MappingFlags,
}

impl MapArea {
    pub fn new(pages: GlobalPage, flags: MappingFlags, start_va: VirtAddr) -> Self {
        Self {
            start_va,
            pages,
            flags,
        }
    }
    pub fn overlap_with(&self, start_va: VirtAddr, end_va: VirtAddr) -> bool {
        return self.start_va <= end_va && self.start_va + self.pages.size() >= start_va;
    }
}
