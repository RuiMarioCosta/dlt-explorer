//! Full-pipeline read throughput (cold buffer).
//!
//! Every method allocates a FRESH destination buffer inside the timed region on
//! every repetition, so the measurement includes allocation, zero-fill-on-demand
//! page faults, the open/close syscalls, and the actual read. This is the
//! "everything is measured" counterpart to `read_warm`.

mod common;
mod profiler;

use common::*;
use memmap2::Mmap;
use profiler::{metrics::*, repetition_tester::*};
use std::ffi::CString;
use std::ffi::c_void;
use std::fs::File;
use std::hint::black_box;

/// `std::fs::read` — opens, allocates, and reads the whole file each rep.
fn read_with_fs_read(tester: &mut RepetitionTester, path: &str) {
    while tester.is_testing() {
        tester.begin();
        let bytes = std::fs::read(path).expect("read file");
        tester.end();
        black_box(&bytes);
        tester.set_accumulated_bytes(bytes.len() as u64);
    }
}

/// C stdio buffered I/O with a fresh buffer allocated each rep.
fn read_with_fread(tester: &mut RepetitionTester, path: &str) {
    let c_path = CString::new(path).expect("path contains a nul byte");
    let mode = CString::new("rb").expect("mode contains a nul byte");

    while tester.is_testing() {
        let size = std::fs::metadata(path).expect("metadata").len() as usize;

        tester.begin();
        let mut buf: Vec<u8> = vec![0u8; size];
        let stream = unsafe { fopen(c_path.as_ptr(), mode.as_ptr()) };
        if stream.is_null() {
            panic!("fopen failed");
        }
        let read = unsafe { fread(buf.as_mut_ptr() as *mut c_void, 1, size, stream) };
        unsafe { fclose(stream) };
        tester.end();

        black_box(&buf);
        tester.set_accumulated_bytes(read as u64);
    }
}

/// Raw Win32 `CreateFileW` + `ReadFile` with a fresh buffer allocated each rep.
fn read_with_win32(tester: &mut RepetitionTester, path: &str) {
    let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

    while tester.is_testing() {
        let size = std::fs::metadata(path).expect("metadata").len() as usize;

        tester.begin();
        let mut buf: Vec<u8> = vec![0u8; size];
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

/// Memory-map the file and touch every byte so the OS actually pages it in.
fn read_with_mmap(tester: &mut RepetitionTester, path: &str) {
    while tester.is_testing() {
        tester.begin();
        let file = File::open(path).expect("open file");
        let mmap = unsafe { Mmap::map(&file).expect("mmap file") };
        let mut sum = 0u64;
        for &byte in mmap.iter() {
            sum += byte as u64;
        }
        black_box(sum);
        tester.end();
        tester.set_accumulated_bytes(mmap.len() as u64);
    }
}

type TestFunction = (&'static str, fn(&mut RepetitionTester, &str));

const TEST_FUNCTIONS: [TestFunction; 4] = [
    ("std::fs::read", read_with_fs_read),
    ("fread", read_with_fread),
    ("Win32 ReadFile", read_with_win32),
    ("Mmap::map", read_with_mmap),
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

    for function in TEST_FUNCTIONS {
        println!("--- {} ---", function.0);
        let mut tester = RepetitionTester::new(file_size, cpu_freq, 10);
        function.1(&mut tester, &path);
    }
}
