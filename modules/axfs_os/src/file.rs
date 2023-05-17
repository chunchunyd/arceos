use alloc::string::{String, ToString};
use super::file_io::FileIO;
use crate::flags::OpenFlags;
use alloc::sync::Arc;
use log::debug;
use axerrno::{AxError, AxResult};
use axfs::api;
use axfs::api::{File, ReadDir};
use axio::{Read, Seek, SeekFrom, Write};
use axsync::Mutex;

/// 文件描述符
pub struct FileDesc {
    /// 文件路径
    pub path: String,
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

    fn get_path(&self) -> String {
        self.path.clone()
    }

    /// debug
    fn print_content(&self) {
        debug!("Into function print_content");
        let mut contents = String::new();
        self.file.lock().read_to_string(&mut contents).unwrap();
        debug!("{}", contents);
    }
}

/// 文件描述符的实现
impl FileDesc {
    /// 创建一个新的文件描述符
    pub fn new(path: &str, file: Arc<Mutex<File>>, flags: OpenFlags) -> Self {
        Self {
            path: path.to_string(),
            file,
            flags,
        }
    }
}

/// 新建一个文件描述符
pub fn new_fd(path: String, flags: OpenFlags) -> AxResult<FileDesc> {
    debug!("Into function new_fd, path: {}", path);
    let mut file = File::options();
    file.read(flags.readable());
    file.write(flags.writable());
    file.create(flags.creatable());
    file.create_new(flags.new_creatable());
    let file = file.open(path.as_str())?;
    // let file_size = file.metadata()?.len();
    let fd = FileDesc::new(path.as_str(), Arc::new(Mutex::new(file)), flags);
    Ok(fd)
}

/// 目录描述符
pub struct DirDesc {
    /// 目录
    pub dir_path:String,
}

/// 目录描述符的实现
impl DirDesc {
    /// 创建一个新的目录描述符
    pub fn new(path: String) -> Self {
        Self {
            dir_path: path,
        }
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

    fn get_path(&self) -> String {
        self.dir_path.to_string().clone()
    }
}

pub fn new_dir(dir_path: String, _flags: OpenFlags) -> AxResult<DirDesc> {
    if let Err(e) = api::read_dir(dir_path.as_str()){
        return Err(e);
    }
    Ok(DirDesc::new(dir_path))
}
