pub struct FileSystemInfo; // TODO

/// File (inode) attribute
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct VfsNodeAttr {
    /// File permission mode.
    mode: VfsNodePerm,
    /// File type.
    ty: VfsNodeType,
    /// Total size, in bytes.
    size: u64,
    /// Number of 512B blocks allocated.
    blocks: u64,
}

bitflags::bitflags! {
    /// File (inode) permission mode.
    #[derive(Debug, Clone, Copy)]
    pub struct VfsNodePerm: u16 {
        /// Owner has read permission.
        const OWNER_READ = 0o400;
        /// Owner has write permission.
        const OWNER_WRITE = 0o200;
        /// Owner has execute permission.
        const OWNER_EXEC = 0o100;

        /// Group has read permission.
        const GROUP_READ = 0o40;
        /// Group has write permission.
        const GROUP_WRITE = 0o20;
        /// Group has execute permission.
        const GROUP_EXEC = 0o10;

        /// Others have read permission.
        const OTHER_READ = 0o4;
        /// Others have write permission.
        const OTHER_WRITE = 0o2;
        /// Others have execute permission.
        const OTHER_EXEC = 0o1;
    }
}

/// File (inode) type.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VfsNodeType {
    /// FIFO (named pipe)
    Fifo = 0o1,
    /// Character device
    CharDevice = 0o2,
    /// Directory
    Dir = 0o4,
    /// Block device
    BlockDevice = 0o6,
    /// Regular file
    File = 0o10,
    /// Symbolic link
    SymLink = 0o12,
    /// Socket
    Socket = 0o14,
}

/// Directory entry.
pub struct VfsDirEntry {
    d_type: VfsNodeType,
    d_name: [u8; 63],
}

impl VfsNodePerm {
    pub const fn default_file() -> Self {
        Self::from_bits_truncate(0o666)
    }

    pub const fn default_dir() -> Self {
        Self::from_bits_truncate(0o755)
    }
}

impl VfsNodeType {
    pub const fn is_file(self) -> bool {
        matches!(self, Self::File)
    }

    pub const fn is_dir(self) -> bool {
        matches!(self, Self::Dir)
    }
}

impl VfsNodeAttr {
    pub const fn new(mode: VfsNodePerm, ty: VfsNodeType, size: u64, blocks: u64) -> Self {
        Self {
            mode,
            ty,
            size,
            blocks,
        }
    }

    pub const fn new_file(size: u64, blocks: u64) -> Self {
        Self {
            mode: VfsNodePerm::default_file(),
            ty: VfsNodeType::File,
            size,
            blocks,
        }
    }

    pub const fn new_dir(size: u64, blocks: u64) -> Self {
        Self {
            mode: VfsNodePerm::default_dir(),
            ty: VfsNodeType::Dir,
            size,
            blocks,
        }
    }

    pub const fn size(&self) -> u64 {
        self.size
    }

    pub const fn file_type(&self) -> VfsNodeType {
        self.ty
    }

    pub const fn is_file(&self) -> bool {
        self.ty.is_file()
    }

    pub const fn is_dir(&self) -> bool {
        self.ty.is_dir()
    }
}

impl VfsDirEntry {
    pub const fn default() -> Self {
        Self {
            d_type: VfsNodeType::File,
            d_name: [0; 63],
        }
    }

    pub fn new(name: &str, ty: VfsNodeType) -> Self {
        let mut d_name = [0; 63];
        if name.len() > d_name.len() {
            log::warn!(
                "directory entry name too long: {} > {}",
                name.len(),
                d_name.len()
            );
        }
        d_name[..name.len()].copy_from_slice(name.as_bytes());
        Self { d_type: ty, d_name }
    }

    pub fn entry_type(&self) -> VfsNodeType {
        self.d_type
    }

    pub fn name_as_bytes(&self) -> &[u8] {
        let len = self
            .d_name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(self.d_name.len());
        &self.d_name[..len]
    }
}
