//! Pure-copy read throughput (warm buffer).
//!
//! Every method reads into the SAME pre-allocated, pre-faulted buffer that
//! `main` hands in. The timed region therefore measures only the copy of file
//! bytes into already-resident memory — no allocation and no zero-fill-on-demand
//! page faults. See `read_cold` for the version that measures the whole pipeline.

mod common;
mod profiler;

use common::*;
use memmap2::Mmap;
use profiler::{metrics::*, repetition_tester::*};
use std::ffi::CString;
use std::ffi::c_void;
use std::fs::File;
use std::hint::black_box;
use std::io::Read;

/// `File::open` + `read_exact` straight into the warm buffer.
fn read_with_fs_open_read(tester: &mut RepetitionTester, buf: &mut [u8], path: &str) {
    while tester.is_testing() {
        let mut file = File::open(path).expect("open file");
        tester.begin();
        file.read_exact(buf).expect("read file");
        tester.end();
        black_box(&buf);
        tester.set_accumulated_bytes(buf.len() as u64);
    }
}

/// C stdio buffered I/O (`fopen` + `fread` + `fclose`) into the warm buffer.
fn read_with_fread(tester: &mut RepetitionTester, buf: &mut [u8], path: &str) {
    let c_path = CString::new(path).expect("path contains a nul byte");
    let mode = CString::new("rb").expect("mode contains a nul byte");
    let size = buf.len();

    while tester.is_testing() {
        let stream = unsafe { fopen(c_path.as_ptr(), mode.as_ptr()) };
        if stream.is_null() {
            panic!("fopen failed");
        }
        tester.begin();
        let read = unsafe { fread(buf.as_mut_ptr() as *mut c_void, 1, size, stream) };
        tester.end();
        unsafe { fclose(stream) };
        black_box(&buf);
        tester.set_accumulated_bytes(read as u64);
    }
}

/// Raw Win32 `CreateFileW` + `ReadFile` into the warm buffer.
fn read_with_win32(tester: &mut RepetitionTester, buf: &mut [u8], path: &str) {
    let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let size = buf.len();

    while tester.is_testing() {
        tester.begin();
        let handle = unsafe {
            CreateFileW(
                wide_path.as_ptr(),
                GENERIC_READ,
                FILE_SHARE_READ,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                std::ptr::null_mut(),
            )
        };
        if handle as isize == -1 {
            panic!("CreateFileW failed");
        }

        // ReadFile caps each call at u32 bytes and may return short, so loop.
        let mut total_read: usize = 0;
        while total_read < size {
            let mut bytes_read: u32 = 0;
            let remaining = (size - total_read).min(u32::MAX as usize) as u32;
            let ok = unsafe {
                ReadFile(
                    handle,
                    buf.as_mut_ptr().add(total_read),
                    remaining,
                    &mut bytes_read,
                    std::ptr::null_mut(),
                )
            };
            if ok == 0 {
                panic!("ReadFile failed");
            }
            if bytes_read == 0 {
                break; // EOF
            }
            total_read += bytes_read as usize;
        }
        unsafe { CloseHandle(handle) };
        tester.end();

        black_box(&buf);
        tester.set_accumulated_bytes(total_read as u64);
    }
}

/// Memory-map the file and `memcpy` it into the warm buffer.
fn read_with_mmap(tester: &mut RepetitionTester, buf: &mut [u8], path: &str) {
    while tester.is_testing() {
        let file = File::open(path).expect("open file");
        let mmap = unsafe { Mmap::map(&file).expect("mmap file") };
        tester.begin();
        buf.copy_from_slice(&mmap);
        tester.end();
        black_box(&buf);
        tester.set_accumulated_bytes(buf.len() as u64);
    }
}

type TestFunction = (&'static str, fn(&mut RepetitionTester, &mut [u8], &str));

const TEST_FUNCTIONS: [TestFunction; 4] = [
    ("open + read_exact", read_with_fs_open_read),
    ("fread", read_with_fread),
    ("Win32 ReadFile", read_with_win32),
    ("Mmap::map + copy", read_with_mmap),
];

fn main() {
    let path = std::env::args().nth(1).expect(
        "usage: <bench> <path-to-file>; pass the file via `cargo bench --bench <name> -- <path>`",
    );
    println!("Test file: {}", path);

    let cpu_freq = get_cpu_frequency();
    println!("Estimated CPU frequency: {} MHz", cpu_freq / 1_000_000);
    let file_size = std::fs::metadata(&path).expect("read metadata").len();
    println!("File size: {} bytes", file_size);

    // Allocate the destination buffer once and write every byte so all pages are
    // faulted in up front. The timed regions then never pay for page faults.
    let mut buf: Vec<u8> = vec![0u8; file_size as usize];
    for byte in buf.iter_mut() {
        *byte = 0;
    }

    for function in TEST_FUNCTIONS {
        println!("--- {} ---", function.0);
        let mut tester = RepetitionTester::new(file_size, cpu_freq, 10);
        function.1(&mut tester, &mut buf, &path);
    }
}
