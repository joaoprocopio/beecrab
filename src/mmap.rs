use libc;
use std::io;
use std::os::fd::AsRawFd;
use std::ptr;
use std::{fs::File, slice};

pub struct Mmap {
    ptr: *const u8,
    len: usize,
}

impl Mmap {
    pub fn map<'a>(file: &File) -> io::Result<&'a [u8]> {
        let len = file.metadata()?.len() as libc::size_t;

        unsafe {
            let ptr = libc::mmap(
                ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            );

            if ptr == libc::MAP_FAILED {
                return Err(io::Error::last_os_error());
            }

            Ok(slice::from_raw_parts(ptr as *const u8, len as usize))
        }
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        // this could fail silently...
        if !self.ptr.is_null() && self.len > 0 {
            unsafe {
                libc::munmap(self.ptr as *mut libc::c_void, self.len);
            }
        }
    }
}
