//! Shared FFI declarations, constants, and the test file path used by the
//! read-throughput benches (`read_pure_copy` and `read_full`).

use std::ffi::{c_char, c_int, c_void};

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn CreateFileW(
        lp_file_name: *const u16,
        dw_desired_access: u32,
        dw_share_mode: u32,
        lp_security_attributes: *mut c_void,
        dw_creation_disposition: u32,
        dw_flags_and_attributes: u32,
        h_template_file: *mut c_void,
    ) -> *mut c_void;
    pub fn ReadFile(
        h_file: *mut c_void,
        lp_buffer: *mut u8,
        n_number_of_bytes_to_read: u32,
        lp_number_of_bytes_read: *mut u32,
        lp_overlapped: *mut c_void,
    ) -> i32;
    pub fn CloseHandle(h_object: *mut c_void) -> i32;
}

pub const GENERIC_READ: u32 = 0x8000_0000;
pub const FILE_SHARE_READ: u32 = 0x0000_0001;
pub const OPEN_EXISTING: u32 = 3;
pub const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;

unsafe extern "C" {
    pub fn fopen(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    pub fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
    pub fn fclose(stream: *mut c_void) -> c_int;
}
