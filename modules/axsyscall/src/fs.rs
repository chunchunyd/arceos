use alloc::string::{String, ToString};
/// 处理关于输出输入的系统调用
use alloc::sync::Arc;
use core::any::Any;
use core::mem::transmute;
use log::{debug, trace};
use memory_addr::{VirtAddr, PhysAddr};
use axfs::api;
use axfs::api::{File, FileType};
use axfs_os::file::{DirDesc, new_dir};
use axfs_os::new_fd;
use axfs_os::pipe::make_pipe;
use axprocess::process::current_process;


#[allow(unused)]
const AT_FDCWD: usize = -100isize as usize;

// const STDIN: usize = 0;
// const STDOUT: usize = 1;
// const STDERR: usize = 2;
/// 功能：从一个文件描述符中读取；
/// 输入：
///     - fd：要读取文件的文件描述符。
///     - buf：一个缓存区，用于存放读取的内容。
///     - count：要读取的字节数。
/// 返回值：成功执行，返回读取的字节数。如为0，表示文件结束。错误，则返回-1。
pub fn syscall_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    debug!("Into syscall_read. fd: {}, buf: {:?}, len: {}", fd, buf as usize, len);
    let process = current_process();
    let process_inner = process.inner.lock();
    let va = VirtAddr::from(buf as usize);
    if fd >= process_inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = process_inner.fd_table[fd].as_ref() {
        if !file.readable() {
            return -1;
        }
        let file = file.clone();

        // // debug
        // file.print_content();

        drop(process_inner); // release current inner manually to avoid multi-borrow
        let read_size = file.read(unsafe { core::slice::from_raw_parts_mut(buf, len) })
            .unwrap() as isize;
        debug!("read_size: {}", read_size);
        read_size as isize
    } else {
        -1
    }
}


/// 功能：从一个文件描述符中写入；
/// 输入：
///     - fd：要写入文件的文件描述符。
///     - buf：一个缓存区，用于存放要写入的内容。
///     - count：要写入的字节数。
/// 返回值：成功执行，返回写入的字节数。错误，则返回-1。
pub fn syscall_write(fd: usize, buf: *const u8, count: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner.lock();
    if fd >= process_inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = process_inner.fd_table[fd].as_ref() {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        drop(process_inner); // release current inner manually to avoid multi-borrow
        // file.write("Test SysWrite\n".as_bytes()).unwrap();
        file.write(unsafe { core::slice::from_raw_parts(buf, count) })
            .unwrap() as isize
    } else {
        -1
    }
}

/// 功能：打开或创建一个文件；
/// 输入：
///     - fd：文件所在目录的文件描述符。
///     - filename：要打开或创建的文件名。如为绝对路径，则忽略fd。如为相对路径，且fd是AT_FDCWD，则filename是相对于当前工作目录来说的。如为相对路径，且fd是一个文件描述符，则filename是相对于fd所指向的目录来说的。
///     - flags：必须包含如下访问模式的其中一种：O_RDONLY，O_WRONLY，O_RDWR。还可以包含文件创建标志和文件状态标志。
///     - mode：文件的所有权描述。详见`man 7 inode `。
/// 返回值：成功执行，返回新的文件描述符。失败，返回-1。
///
/// 说明：如果打开的是一个目录，那么返回的文件描述符指向的是该目录的描述符。(后面会用到针对目录的文件描述符)
/// flags: O_RDONLY: 0, O_WRONLY: 1, O_RDWR: 2
pub fn syscall_openat(fd: usize, path: *const u8, flags: usize, _mode: u8) -> isize {
    debug!("Into syscall_open. fd: {}, path: {}, flags: {}, mode: {}", fd, path as usize, flags, _mode);

    let process = current_process();
    let mut process_inner = process.inner.lock();
    let path = process_inner.memory_set.lock().translate_str(path);

    let fd_num = process_inner.alloc_fd();
    debug!("allocated fd_num: {}", fd_num);

    if let Ok(file) = new_fd(path.as_str(), flags.into()) {
        debug!("new file_desc successfully allocated");
        process_inner.fd_table[fd_num] = Some(Arc::new(file));
        fd_num as isize
    } else if let Ok(dir) = new_dir(path.to_string()) {
        debug!("new dir_desc successfully allocated");
        process_inner.fd_table[fd_num] = Some(Arc::new(dir));
        fd_num as isize
    } else {
        debug!("open file/dir failed");
        -1
    }
}

/// 功能：关闭一个文件描述符；
/// 输入：
///     - fd：要关闭的文件描述符。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_close(fd: usize) -> isize {
    debug!("Into syscall_close. fd: {}", fd);

    let process = current_process();
    let mut process_inner = process.inner.lock();

    for i in 0..process_inner.fd_table.len() {
        if let Some(file) = process_inner.fd_table[i].as_ref() {
            debug!("fd: {} has file", i);
        }
    }

    if fd >= process_inner.fd_table.len() {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    // if fd == 3 {
    //     debug!("fd {} is reserved for cwd", fd);
    //     return -1;
    // }
    if process_inner.fd_table[fd].is_none() {
        debug!("fd {} is not opened", fd);
        return -1;
    }
    process_inner.fd_table[fd].take();

    for i in 0..process_inner.fd_table.len() {
        if let Some(file) = process_inner.fd_table[i].as_ref() {
            debug!("fd: {} has file", i);
        }
    }

    0
}

/// 功能：获取当前工作目录；
/// 输入：
///     - char *buf：一块缓存区，用于保存当前工作目录的字符串。当buf设为NULL，由系统来分配缓存区。
///     - size：buf缓存区的大小。
/// 返回值：成功执行，则返回当前工作目录的字符串的指针。失败，则返回NULL。
///  暂时：成功执行，则返回当前工作目录的字符串的指针 as isize。失败，则返回-1。
pub fn syscall_getcwd(mut buf: *mut u8, len: usize) -> isize {
    debug!("Into syscall_getcwd. buf: {}, len: {}", buf as usize, len);
    // let process = current_process();
    // let process_inner = process.inner.lock();
    // let cwd = process_inner.get_cwd();
    let cwd = api::current_dir().unwrap();

    // todo: 如果buf为NULL,则系统分配缓存区
    // if buf.is_null() {
    //     buf = allocate_buffer(cwd.len());   // 分配缓存区 allocate_buffer
    // }

    let cwd = cwd.as_bytes();
    let len = core::cmp::min(len, cwd.len());
    unsafe {
        core::ptr::copy_nonoverlapping(cwd.as_ptr(), buf, len);
    }
    // 返回当前工作目录的字符串的指针
    if len == cwd.len() {   // 如果buf的长度足够大
        // buf as *const u8
        buf as isize
    } else {
        debug!("getcwd: buf size is too small");
        // core::ptr::null()
        -1
    }
}

/// 功能：创建管道；
/// 输入：
///     - fd[2]：用于保存2个文件描述符。其中，fd[0]为管道的读出端，fd[1]为管道的写入端。
/// 返回值：成功执行，返回0。失败，返回-1。
///
/// 注意：fd[2]是32位数组，所以这里的 fd 是 u32 类型的指针，而不是 usize 类型的指针。
pub fn syscall_pipe2(fd: *mut u32) -> isize {
    debug!("Into syscall_pipe2. fd: {}", fd as usize);
    let process = current_process();
    let mut process_inner = process.inner.lock();

    let (read, write) = make_pipe();

    let fd_num = process_inner.alloc_fd();
    process_inner.fd_table[fd_num] = Some(read);
    let fd_num2 = process_inner.alloc_fd();
    process_inner.fd_table[fd_num2] = Some(write);


    debug!("fd_num: {}, fd_num2: {}", fd_num, fd_num2);

    unsafe {
        core::ptr::write(fd, fd_num as u32);
        core::ptr::write(fd.offset(1), fd_num2 as u32);
    }
    0
}

/// 功能：复制文件描述符；
/// 输入：
///     - fd：被复制的文件描述符。
/// 返回值：成功执行，返回新的文件描述符。失败，返回-1。
pub fn syscall_dup(fd: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner.lock();

    if fd >= process_inner.fd_table.len() {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    if process_inner.fd_table[fd].is_none() {
        debug!("fd {} is a closed fd", fd);
        return -1;
    }

    let fd_num = process_inner.alloc_fd();
    process_inner.fd_table[fd_num] = process_inner.fd_table[fd].clone();

    fd_num as isize
}

/// 功能：复制文件描述符，并指定了新的文件描述符；
/// 输入：
///     - old：被复制的文件描述符。
///     - new：新的文件描述符。
/// 返回值：成功执行，返回新的文件描述符。失败，返回-1。
pub fn syscall_dup3(fd: usize, new_fd: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner.lock();

    if fd >= process_inner.fd_table.len() {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    if process_inner.fd_table[fd].is_none() {
        debug!("fd {} is not opened", fd);
        return -1;
    }
    if new_fd >= process_inner.fd_table.len() {
        for i in process_inner.fd_table.len()..new_fd + 1 {
            process_inner.fd_table.push(None);
        }
    }
    if process_inner.fd_table[new_fd].is_some() {
        debug!("new_fd {} is already opened", new_fd);
        return -1;
    }
    process_inner.fd_table[new_fd] = process_inner.fd_table[fd].clone();

    new_fd as isize
}

/// 功能：创建目录；
/// 输入：
///     - dirfd：要创建的目录所在的目录的文件描述符。
///     - path：要创建的目录的名称。如果path是相对路径，则它是相对于dirfd目录而言的。如果path是相对路径，且dirfd的值为AT_FDCWD，则它是相对于当前路径而言的。如果path是绝对路径，则dirfd被忽略。
///     - mode：文件的所有权描述。详见`man 7 inode `。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_mkdirat(dirfd: usize, path: *const u8, mode: u32) -> isize {
    let process = current_process();
    let mut process_inner = process.inner.lock();
    let path = process_inner.memory_set.lock().translate_str(path);
    debug!("Into syscall_mkdirat. dirfd: {}, path: {:?}, mode: {}", dirfd, path, mode);
    // debug!("current dir: {}", api::current_dir().unwrap());
    // 如果 path是绝对路径 或 dirfd是AT_FDCWD
    if path.starts_with('/') || dirfd == AT_FDCWD {
        let res = api::create_dir(path.as_str());
        if res.is_ok() {
            0
        } else {
            -1
        }
    }
    else {
        // TODO 支持dirfd
        // if dirfd >= process_inner.fd_table.len() {} else if process_inner.fd_table[dirfd].is_none() {
        //     debug!("dirfd {} is not opened", dirfd);
        //     return -1;
        // }
        // let dir = process_inner.fd_table[dirfd].as_ref().unwrap();
        // if let Some(dir) = dir.as_ref().downcast_ref::<DirDesc>() {
        //     let mut dir_inner = dir.inner.lock();
        //     let res = api::create_dir(path.as_str());
        //     0
        // } else {
        //     debug!("dirfd {} is not a dir", dirfd);
        //     -1
        // }
        -1
    }
}


/// 功能：切换工作目录；
/// 输入：
///     - path：需要切换到的目录。
/// 返回值：成功执行，返回0。失败，返回-1。
pub fn syscall_chdir(path: *const u8) -> isize {
    // 从path中读取字符串
    let process = current_process();
    let mut process_inner = process.inner.lock();
    let path = process_inner.memory_set.lock().translate_str(path);

    let res = api::set_current_dir(path.as_str());

    if res.is_err() {
        debug!("chdir failed");
        return -1;
    } else {
        0
    }
}

/// 返回类型
struct Dirent {
    // 索引结点号
    d_ino: u64,
    // 到下一个dirent的偏移
    d_off: i64,
    // 当前dirent的长度
    d_reclen: u16,
    // 文件类型
    d_type: u8,
    //文件名
    d_name: [u8],
}
/// 功能：获取目录的条目;
/// 参数：
///     -fd：所要读取目录的文件描述符。
///     -buf：一个缓存区，用于保存所读取目录的信息。缓存区的结构如下
/// struct dirent {
///     uint64 d_ino;	// 索引结点号
///     int64 d_off;	// 到下一个dirent的偏移
///     unsigned short d_reclen;	// 当前dirent的长度
///     unsigned char d_type;	// 文件类型
///     char d_name[];	//文件名
/// };
/// 返回值：成功执行，返回读取的字节数。当到目录结尾，则返回0。失败，则返回-1。
pub fn syscall_getdents64(fd: usize, buf: *mut u8) -> isize {
    let process = current_process();
    let mut process_inner = process.inner.lock();

    if fd >= process_inner.fd_table.len() {
        debug!("fd {} is out of range", fd);
        return -1;
    }
    if process_inner.fd_table[fd].is_none() {
        debug!("fd {} is not opened", fd);
        return -1;
    }

    let dir = process_inner.fd_table[fd].as_ref().unwrap();
    let mut file_inner = file.inner.lock();

    let mut buf = unsafe { core::slice::from_raw_parts_mut(buf, 1024) };
    let mut offset = 0;
    let mut cnt = 0;
    loop {
        let mut entry = file_inner.dir_entry(offset);
        if entry.is_none() {
            break;
        }
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name = name.as_bytes();
        let name_len = name.len();
        let entry_size = 24 + name_len;
        if buf.len() - cnt < entry_size {
            break;
        }
        unsafe {
            core::ptr::write(buf.as_mut_ptr().offset(cnt as isize) as *mut u64, entry.inode() as u64);
            core::ptr::write(buf.as_mut_ptr().offset(cnt as isize + 8) as *mut i64, offset as i64 + entry_size as i64);
            core::ptr::write(buf.as_mut_ptr().offset(cnt as isize + 16) as *mut u16, entry_size as u16);
            core::ptr::write(buf.as_mut_ptr().offset(cnt as isize + 18) as *mut u8, entry.file_type() as u8);
            core::ptr::copy_nonoverlapping(name.as_ptr(), buf.as_mut_ptr().offset(cnt as isize + 24), name_len);
        }
        cnt += entry_size;
        offset += entry_size;
    }
    cnt as isize
}

// /// 功能：创建文件的链接；
// /// 输入：
// ///     - olddirfd：原来的文件所在目录的文件描述符。
// ///     - oldpath：文件原来的名字。如果oldpath是相对路径，则它是相对于olddirfd目录而言的。如果oldpath是相对路径，且olddirfd的值为AT_FDCWD，则它是相对于当前路径而言的。如果oldpath是绝对路径，则olddirfd被忽略。
// ///     - newdirfd：新文件名所在的目录。
// ///     - newpath：文件的新名字。newpath的使用规则同oldpath。
// ///     - flags：在2.6.18内核之前，应置为0。其它的值详见`man 2 linkat`。
// /// 返回值：成功执行，返回0。失败，返回-1。
// pub fn sys_linkat(olddirfd: usize, oldpath: *const u8, newdirfd: usize, newpath: *const u8, flags: usize) -> isize {
//     let process = current_process();
//     let mut process_inner = process.inner.lock();
//
//     if olddirfd >= process_inner.fd_table.len() {
//         debug!("olddirfd {} is out of range", olddirfd);
//         return -1;
//     }
//     if process_inner.fd_table[olddirfd].is_none() {
//         debug!("olddirfd {} is not opened", olddirfd);
//         return -1;
//     }
//     if newdirfd >= process_inner.fd_table.len() {
//         debug!("newdirfd {} is out of range", newdirfd);
//         return -1;
//     }
//     if process_inner.fd_table[newdirfd].is_none() {
//         debug!("newdirfd {} is not opened", newdirfd);
//         return -1;
//     }
//
//     let raw_str = unsafe { core::slice::from_raw_parts(oldpath, 256) };
//     let mut len = 0 as usize;
//     for i in 0..256 {
//         if raw_str[i] == 0 {
//             len = i;
//             break;
//         }
//         if i == 255 {
//             debug!("oldpath is too long");
//             return -1;
//         }
//     }
//     let oldpath = unsafe { core::slice::from_raw_parts(oldpath, len) };
//     let oldpath = core::str::from_utf8(oldpath).unwrap();
//
//     let raw_str = unsafe { core::slice::from_raw_parts(newpath, 256) };
//     let mut len = 0 as usize;
//     for i in 0..256 {
//         if newpath[i] == 0 {
//             len = i;
//             break;
//         }
//         if i == 255 {
//             debug!("newpath is too long");
//             return -1;
//         }
//     }
//     let newpath = unsafe { core::slice::from_raw_parts(newpath, len) };
//     let newpath = core::str::from_utf8(newpath).unwrap();
//
//     let olddir = process_inner.fd_table[olddirfd].as_ref().unwrap();
//     let mut olddir_inner = olddir.inner.lock();
//     let newdir = process_inner.fd_table[newdirfd].as_ref().unwrap();
//     let mut newdir_inner = newdir.inner.lock();
//
//     let oldfile = olddir_inner.find(oldpath);
//     if oldfile.is_none() {
//         debug!("oldpath {} is not found", oldpath);
//         return -1;
//     }
//     let oldfile = oldfile.unwrap();
//     let oldfile_inner = oldfile.inner.lock();
//     if oldfile_inner.file_type() != FileType::File {
//         debug!("oldpath {} is not a file", oldpath);
//         return -1;
//     }
//
//     let newfile = newdir_inner.find(newpath);
//     if newfile.is_some() {
//         debug!("newpath {} is already exist", newpath);
//         return -1;
//     }
//
//     let newfile = oldfile.clone();
//     let mut newfile_inner = newfile.inner.lock();
//     newfile_inner.set_name(newpath);
//     newdir_inner.add(newfile);
//     0
// }
//
// /// 功能：移除指定文件的链接(可用于删除文件)；
// /// 输入：
// ///     - dirfd：要删除的链接所在的目录。
// ///     - path：要删除的链接的名字。如果path是相对路径，则它是相对于dirfd目录而言的。如果path是相对路径，且dirfd的值为AT_FDCWD，则它是相对于当前路径而言的。如果path是绝对路径，则dirfd被忽略。
// ///     - flags：可设置为0或AT_REMOVEDIR。
// /// 返回值：成功执行，返回0。失败，返回-1。
// pub fn syscall_unlinkat(dirfd: usize, path: *const u8, flags: usize) -> isize {
//     let process = current_process();
//     let mut process_inner = process.inner.lock();
//
//     if dirfd >= process_inner.fd_table.len() {
//         debug!("dirfd {} is out of range", dirfd);
//         return -1;
//     }
//     if process_inner.fd_table[dirfd].is_none() {
//         debug!("dirfd {} is not opened", dirfd);
//         return -1;
//     }
//
//     let raw_str = unsafe { core::slice::from_raw_parts(path, 256) };
//     let mut len = 0 as usize;
//     for i in 0..256 {
//         if raw_str[i] == 0 {
//             len = i;
//             break;
//         }
//         if i == 255 {
//             debug!("path is too long");
//             return -1;
//         }
//     }
//     let path = unsafe { core::slice::from_raw_parts(path, len) };
//     let path = core::str::from_utf8(path).unwrap();
//
//     let dir = process_inner.fd_table[dirfd].as_ref().unwrap();
//     let mut dir_inner = dir.inner.lock();
//
//     let file = dir_inner.find(path);
//     if file.is_none() {
//         debug!("path {} is not found", path);
//         return -1;
//     }
//     let file = file.unwrap();
//     let mut file_inner = file.inner.lock();
//     if file_inner.file_type() == FileType::Dir {
//         if flags != 0 {
//             dir_inner.remove(path);
//             0
//         } else {
//             debug!("path {} is a directory", path);
//             return -1;
//         }
//     } else {
//         dir_inner.remove(path);
//         0
//     }
// }
//

//
// /// 功能：卸载文件系统；
// /// 输入：指定卸载目录，卸载参数；
// /// 返回值：成功返回0，失败返回-1；
// pub fn syscall_umount(dirfd: usize, flags: usize) -> isize {
//     let process = current_process();
//     let mut process_inner = process.inner.lock();
//
//     if dirfd >= process_inner.fd_table.len() {
//         debug!("dirfd {} is out of range", dirfd);
//         return -1;
//     }
//     if process_inner.fd_table[dirfd].is_none() {
//         debug!("dirfd {} is not opened", dirfd);
//         return -1;
//     }
//
//     let dir = process_inner.fd_table[dirfd].as_ref().unwrap();
//     let mut dir_inner = dir.inner.lock();
//
//     if dir_inner.file_type() != FileType::Dir {
//         debug!("dirfd {} is not a directory", dirfd);
//         return -1;
//     }
//
//     let fs = dir_inner.fs();
//     if fs.is_none() {
//         debug!("dirfd {} is not a mount point", dirfd);
//         return -1;
//     }
//     let fs = fs.unwrap();
//     let mut fs_inner = fs.inner.lock();
//
//     if flags != 0 {
//         fs_inner.umount();
//         dir_inner.set_fs(None);
//         0
//     } else {
//         debug!("dirfd {} is not a mount point", dirfd);
//         return -1;
//     }
// }
//
// /// 功能：挂载文件系统；
// /// 输入：
// ///   - special: 挂载设备；
// ///   - dir: 挂载点；
// ///   - fstype: 挂载的文件系统类型；
// ///   - flags: 挂载参数；
// ///   - data: 传递给文件系统的字符串参数，可为NULL；
// /// 返回值：成功返回0，失败返回-1
// pub fn syscall_mount(
//     special: *const u8,
//     dir: *const u8,
//     fstype: *const u8,
//     flags: usize,
//     data: *const u8,
// ) -> isize {
//     let process = current_process();
//     let mut process_inner = process.inner.lock();
//
//     let raw_str = unsafe { core::slice::from_raw_parts(special, 256) };
//     let mut len = 0 as usize;
//     for i in 0..256 {
//         if raw_str[i] == 0 {
//             len = i;
//             break;
//         }
//         if i == 255 {
//             debug!("special is too long");
//             return -1;
//         }
//     }
//     let special = unsafe { core::slice::from_raw_parts(special, len) };
//     let special = core::str::from_utf8(special).unwrap();
//
//     let raw_str = unsafe { core::slice::from_raw_parts(dir, 256) };
//     let mut len = 0 as usize;
//     for i in 0..256 {
//         if raw_str[i] == 0 {
//             len = i;
//             break;
//         }
//         if i == 255 {
//             debug!("dir is too long");
//             return -1;
//         }
//     }
//     let dir = unsafe { core::slice::from_raw_parts(dir, len) };
//     let dir = core::str::from_utf8(dir).unwrap();
//
//     let raw_str = unsafe { core::slice::from_raw_parts(fstype, 256) };
//     let mut len = 0 as usize;
//     for i in 0..256 {
//         if raw_str[i] == 0 {
//             len = i;
//             break;
//         }
//         if i == 255 {
//             debug!("fstype is too long");
//             return -1;
//         }
//     }
//     let fstype = unsafe { core::slice::from_raw_parts(fstype, len) };
//     let fstype = core::str::from_utf8(fstype).unwrap();
//
//     let raw_str = unsafe { core::slice::from_raw_parts(data, 256) };
//     let mut len = 0 as usize;
//     for i in 0..256 {
//         if raw_str[i] == 0 {
//             len = i;
//             break;
//         }
//         if i == 255 {
//             debug!("data is too long");
//             return -1;
//         }
//     }
//     let data = unsafe { core::slice::from_raw_parts(data, len) };
//     let data = core::str::from_utf8(data).unwrap();
//
//     let fs = File::new(special, FileType::Dir, 0);
//     let fs = fs.open();
//     if fs.is_none() {
//         debug!("special {} is not exist", special);
//         return -1;
//     }
//     let fs = fs.unwrap();
//     let mut fs_inner = fs.inner.lock();
//
//     let dir = process_inner.find(dir);
//     if dir.is_none() {
//         debug!("dir {} is not exist", dir);
//         return -1;
//     }
//     let dir = dir.unwrap();
//     let mut dir_inner = dir.inner.lock();
//
//     if dir_inner.file_type() != FileType::Dir {
//         debug!("dir {} is not a directory", dir);
//         return -1;
//     }
//
//     if dir_inner.fs().is_some() {
//         debug!("dir {} is already mounted", dir);
//         return -1;
//     }
//
//     fs_inner.mount(dir, fstype, flags, data);
//     dir_inner.set_fs(Some(fs));
//     0
// }
//
// /// 功能：获取文件状态；
// /// 输入：
// ///     - fd: 文件句柄；
// ///     - kst: 接收保存文件状态的指针；
// /// 返回值：成功返回0，失败返回-1；
// pub fn syscall_fstat(fd: usize, kst: *mut KStat) -> isize {
//     let process = current_process();
//     let mut process_inner = process.inner.lock();
//
//     if fd >= process_inner.fd_table.len() {
//         debug!("fd {} is out of range", fd);
//         return -1;
//     }
//     if process_inner.fd_table[fd].is_none() {
//         debug!("fd {} is not opened", fd);
//         return -1;
//     }
//
//     let file = process_inner.fd_table[fd].as_ref().unwrap();
//     let file_inner = file.inner.lock();
//
//     let mut st = KStat::default();
//     st.st_dev = file_inner.dev();
//     st.st_ino = file_inner.ino();
//     st.st_mode = file_inner.mode();
//     st.st_nlink = file_inner.nlink();
//     st.st_uid = file_inner.uid();
//     st.st_gid = file_inner.gid();
//     st.st_rdev = file_inner.rdev();
//     st.st_size = file_inner.size();
//     st.st_blksize = file_inner.blksize();
//     st.st_blocks = file_inner.blocks();
//     st.st_atime_sec = file_inner.atime_sec();
//     st.st_atime_nsec = file_inner.atime_nsec();
//     st.st_mtime_sec = file_inner.mtime_sec();
//     st.st_mtime_nsec = file_inner.mtime_nsec();
//     st.st_ctime_sec = file_inner.ctime_sec();
//     st.st_ctime_nsec = file_inner.ctime_nsec();
//
//     unsafe {
//         *kst = st;
//     }
//     0
// }