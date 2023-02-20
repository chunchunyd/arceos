use crate::{PageTable64, PagingMetaData};
use page_table_entry::x86_64::X64PTE;

pub struct X64PagingMetaData;

impl const PagingMetaData for X64PagingMetaData {
    const LEVELS: usize = 3;
    const PA_MAX_BITS: usize = 52;
    const VA_MAX_BITS: usize = 48;
}

pub type X64PageTable<I> = PageTable64<X64PagingMetaData, X64PTE, I>;
