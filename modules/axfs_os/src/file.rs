use alloc::string::String;
use super::file_io::FileIO;
use crate::flags::OpenFlags;
use alloc::sync::Arc;
use log::debug;
use axerrno::{AxError, AxResult};
use axfs::api::File;
use axio::{Read, Seek, SeekFrom, Write};
use axsync::Mutex;

/// 文件描述符
pub struct FileDesc {
    /// 文件
    pub file: Arc<Mutex<File>>,
    /// 文件打开的标志位
    pub flags: OpenFlags,
}

/// 为FileDesc实现FileIO trait
impl FileIO for FileDesc {
    fn readable(&self) -> bool {
        self.flags.readable()
    }

    fn writable(&self) -> bool {
        self.flags.writable()
    }

    fn read(&self, buf: &mut [u8]) -> AxResult<usize> {
        debug!("Into function read, buf_len: {}", buf.len());
        self.file.lock().read(buf)
    }

    fn write(&self, buf: &[u8]) -> AxResult<usize> {
        self.file.lock().write(buf)
    }

    fn seek(&self, offset: usize) -> AxResult<u64> {
        self.file.lock().seek(SeekFrom::Start(offset as u64))
    }
    /// debug
    fn print_content(&self){
        debug!("Into function print_content");
        let mut contents = String::new();
        self.file.lock().read_to_string(&mut contents).unwrap();
        debug!("{}", contents);
    }
}

/// 文件描述符的实现
impl FileDesc {
    /// 创建一个新的文件描述符
    pub fn new(file: Arc<Mutex<File>>, flags: OpenFlags) -> Self {
        Self {
            file,
            flags,
        }
    }
}

/// 新建一个文件描述符
pub fn new_fd(path: &str, flags: OpenFlags) -> AxResult<FileDesc> {
    debug!("Into function new_fd, path: {}", path);
    let mut file = File::options();
    file.read(flags.readable());
    file.write(flags.writable());
    file.create(flags.creatable());
    file.create_new(flags.new_creatable());
    let file = file.open(path)?;
    // let file_size = file.metadata()?.len();
    let fd = FileDesc::new(Arc::new(Mutex::new(file)), flags);
    Ok(fd)
}

/// 目录描述符
pub struct DirDesc {
    /// 目录
    //TODO: work_dir: Arc<Mutex<Directory>>, 支持更复杂的操作
    //TODO: 现在新建时任何输入string都不会出错，需要检查输入的路径是否是目录
    pub inner: String,
}

/// 工作目录描述符的实现
impl DirDesc {
    pub fn new(dir_path: String) -> Self {
        Self { inner: dir_path }
    }
    /// 获取工作目录
    pub fn get_path(&self) -> String {
        self.inner.clone()
    }
}

/// 为DirDesc实现FileIO trait
impl FileIO for DirDesc {
    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        false
    }

    fn read(&self, _buf: &mut [u8]) -> AxResult<usize> {
        Err(AxError::IsADirectory)
    }

    fn write(&self, _buf: &[u8]) -> AxResult<usize> {
        Err(AxError::IsADirectory)
    }
}

pub fn new_dir(dir_path: String) -> AxResult<DirDesc> {
    Ok(DirDesc::new(dir_path))
}