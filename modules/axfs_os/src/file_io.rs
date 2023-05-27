use alloc::string::String;
use core::any::Any;
use log::debug;
use axerrno::{AxError, AxResult};
use crate::{DirDesc, FileDesc, Pipe, Stderr, Stdin, Stdout};
use crate::types::Kstat;

/// 文件类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileIOType {
    /// 文件
    FileDesc,
    /// 目录
    DirDesc,
    /// 标准输入输出错误流
    Stdin,
    Stdout,
    Stderr,
    /// 管道
    Pipe,
    /// 链接
    Link,
    /// 其他
    Other,
}

// /// 文件类型
// pub enum ReturnType<'a> {
//     /// 文件
//     FileDesc(&'a FileDesc),
//     /// 目录
//     DirDesc(&'a DirDesc),
//     /// 标准输入输出错误流
//     Stdin(&'a Stdin),
//     Stdout(&'a Stdout),
//     Stderr(&'a Stderr),
//     /// 管道
//     Pipe(&'a Pipe),
//     /// 链接
//     Link(),
//     /// 其他
//     Other(),
// }

/// File I/O trait. 文件I/O操作
pub trait FileIO: Send + Sync + AsAny {
    /// 文件是否可读
    fn readable(&self) -> bool;
    /// 文件是否可写
    fn writable(&self) -> bool;
    /// 读取文件数据到缓冲区, 返回读取的字节数
    fn read(&self, buf: &mut [u8]) -> AxResult<usize>;
    /// 将缓冲区数据写入文件, 返回写入的字节数
    fn write(&self, buf: &[u8]) -> AxResult<usize>;
    /// 移动文件指针, 返回新的文件指针位置
    fn seek(&self, _pos: usize) -> AxResult<u64> {
        Err(AxError::Unsupported) // 如果没有实现seek, 则返回Unsupported
    }
    /// 刷新文件缓冲区
    fn flush(&self) -> AxResult<()> {
        Err(AxError::Unsupported) // 如果没有实现flush, 则返回Unsupported
    }
    /// 获取路径
    fn get_path(&self) -> String {
        debug!("Function get_path not implemented");
        String::from("Function get_path not implemented")
    }
    /// 获取文件信息
    fn get_stat(&self) -> AxResult<Kstat> {
        Err(AxError::Unsupported) // 如果没有实现get_stat, 则返回Unsupported
    }
    /// 获取类型
    fn get_type(&self) -> FileIOType;
    // /// 转换为原有类型的引用
    // fn get_downcast_ref<'a>(&self) -> ReturnType<'a> {
    //     match self.get_type() {
    //         FileIOType::FileDesc => ReturnType::FileDesc(self.as_any().downcast_ref::<FileDesc>().unwrap().clone()),
    //         FileIOType::DirDesc => ReturnType::DirDesc(self.as_any().downcast_ref::<DirDesc>().unwrap().clone()),
    //         FileIOType::Stdin => ReturnType::Stdin(self.as_any().downcast_ref::<Stdin>().unwrap().clone()),
    //         FileIOType::Stdout => ReturnType::Stdout(self.as_any().downcast_ref::<Stdout>().unwrap().clone()),
    //         FileIOType::Stderr => ReturnType::Stderr(self.as_any().downcast_ref::<Stderr>().unwrap().clone()),
    //         FileIOType::Pipe => ReturnType::Pipe(self.as_any().downcast_ref::<Pipe>().unwrap().clone()),
    //         FileIOType::Link => ReturnType::Link,
    //         FileIOType::Other => ReturnType::Other,
    //     }
    // }
    /// debug
    fn print_content(&self) {
        debug!("Function print_content not implemented");
    }
}

/// `FileIO` 需要满足 `AsAny` 的要求，即可以转化为 `Any` 类型，从而能够进行向下类型转换。
pub trait AsAny {
    /// 把当前对象转化为 `Any` 类型，供后续 downcast 使用
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any { self }
}
