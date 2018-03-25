use {Status,Result,ucs2};
use core::mem;

pub const FILE_MODE_READ    : u64 = 0x0000000000000001;
pub const FILE_MODE_WRITE   : u64 = 0x0000000000000002;
pub const FILE_MODE_CREATE  : u64 = 0x8000000000000000;

pub const FILE_READ_ONLY    : u64 = 0x0000000000000001;
pub const FILE_HIDDEN       : u64 = 0x0000000000000002;
pub const FILE_SYSTEM       : u64 = 0x0000000000000004;
pub const FILE_RESERVED     : u64 = 0x0000000000000008;
pub const FILE_DIRECTORY    : u64 = 0x0000000000000010;
pub const FILE_ARCHIVE      : u64 = 0x0000000000000020;
pub const FILE_VALID_ATTR   : u64 = 0x0000000000000037;

#[repr(C)]
pub struct File {
    revision: u64,
    open: extern "C" fn(this: &mut File, new_handle: &mut usize, filename: *const u16, open_mode: u64, attributes: u64) -> Status,
    close: extern "C" fn(this: &mut File) -> Status,
    delete: extern "C" fn(this: &mut File) -> Status,
    read: extern "C" fn(this: &mut File, buffer_size: &mut usize, buffer: *mut u8) -> Status,
    write: extern "C" fn(this: &mut File, buffer_size: &mut usize, buffer: *const u8) -> Status,
    get_position: extern "C" fn(this: &mut File, position: &mut u64) -> Status,
    set_position: extern "C" fn(this: &mut File, position: u64) -> Status,
    get_info: usize,
    set_info: usize,
    flush: extern "C" fn(this: &mut File) -> Status,
}

#[repr(C)]
pub struct SimpleFileSystem {
    revision: u64,
    open_volume: extern "C" fn(this: &mut SimpleFileSystem, root: &mut usize) -> Status, 
}

impl File {
    pub fn open(&mut self, filename: &str, open_mode: u64, attributes: u64) -> Result<*mut File> {
        const BUF_SIZE : usize = 128;
        if filename.len() > BUF_SIZE {
            Err(Status::InvalidParameter)
        }
        else {
            let mut buf = [0u16; BUF_SIZE+1];
            let mut ptr = 0usize;
            match ucs2::encode_ucs2(filename, &mut buf) {
                Ok(_) => { (self.open)(self, &mut ptr, buf.as_ptr(), open_mode, attributes).into_with(|| ptr as *mut File) },
                Err(err) => { Err(err) },
            }
        }
    }
    pub fn close(&mut self) -> Result<()> {
        (self.close)(self).into()
    }
    pub fn delete(&mut self) -> Result<()> {
        (self.delete)(self).into()
    }
    pub fn read(&mut self, buffer: &mut[u8]) -> Result<usize> {
        let mut buffer_size = buffer.len();
        (self.read)(self, &mut buffer_size, buffer.as_mut_ptr()).into_with(|| buffer_size)
    }
    pub fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        let mut buffer_size = buffer.len();
        (self.write)(self, &mut buffer_size, buffer.as_ptr()).into_with(|| buffer_size)
    }
    pub fn get_position(&mut self) -> Result<u64> {
        let mut pos = 0u64;
        (self.get_position)(self, &mut pos).into_with(|| pos)
    }
    pub fn set_position(&mut self, position: u64) -> Result<()> {
        (self.set_position)(self, position).into()
    }
    pub fn flush(&mut self) -> Result<()> {
        (self.flush)(self).into()
    }
}

impl SimpleFileSystem {
    pub fn open_volume(&mut self) -> Result<*mut File> {
        let mut ptr = 0usize;
        (self.open_volume)(self, &mut ptr).into_with(|| ptr as *mut File)
    }
}

impl_proto! {
    protocol SimpleFileSystem {
        GUID = 0x0964e5b22,0x6459,0x11d2,[0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b];
    }
}
