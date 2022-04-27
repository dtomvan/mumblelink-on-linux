extern crate kernel32;
extern crate winapi;

use libc::{c_void, wchar_t};

pub fn copy(dest: &mut [wchar_t], src: &str) {
    if dest.is_empty() { return }
    let mut index = 0;
    for ch in src.encode_utf16() {
        if index == dest.len() - 1 { break }
        dest[index] = ch;
        index += 1;
    }
    dest[index] = 0;
}

pub fn read(src: &[wchar_t]) -> String {
    let zero = src.iter().position(|&c| c == 0).unwrap_or(src.len());
    String::from_utf16_lossy(&src[..zero])
}

pub struct Map {
    handle: winapi::HANDLE,
    pub ptr: *mut c_void,
}

impl Map {
    pub fn new(size: usize) -> Result<Map, super::ErrorCode> {
        unsafe {
            let handle = kernel32::OpenFileMappingW(
                winapi::FILE_MAP_ALL_ACCESS,
                winapi::FALSE,
                wide!(M u m b l e L i n k).as_ptr(),
            );
            if handle.is_null() {
                return Err(super::ErrorCode::OpenFileMappingW);
            }
            let ptr = kernel32::MapViewOfFile(
                handle,
                winapi::FILE_MAP_ALL_ACCESS,
                0,
                0,
                size as u64,
            );
            if ptr.is_null() {
                kernel32::CloseHandle(handle);
                return Err(super::ErrorCode::MapViewOfFile);
            }
            Ok(Map {
                handle,
                ptr: ptr as *mut c_void,
            })
        }
    }
}

impl Drop for Map {
    fn drop(&mut self) {
        unsafe {
            kernel32::CloseHandle(self.handle);
        }
    }
}
