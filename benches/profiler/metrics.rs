#![allow(dead_code)]

use std::arch::x86_64::_rdtsc;
use std::time::Instant;

#[repr(C)]
struct ProcessMemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn QueryPerformanceFrequency(freq: *mut i64) -> i32;
    fn QueryPerformanceCounter(count: *mut i64) -> i32;
    fn GetCurrentProcess() -> isize;
    fn K32GetProcessMemoryInfo(
        process: isize,
        counters: *mut ProcessMemoryCounters,
        cb: u32,
    ) -> i32;
}

/// Ticks per second of the OS performance counter.
#[inline]
pub fn os_timer_frequency() -> i64 {
    let mut freq = 0i64;
    unsafe { QueryPerformanceFrequency(&mut freq) };
    freq
}

/// Current OS performance counter value (in ticks).
#[inline]
pub fn os_timer() -> i64 {
    let mut count = 0i64;
    unsafe { QueryPerformanceCounter(&mut count) };
    count
}

#[inline(always)]
pub fn read_cpu_timer() -> u64 {
    unsafe { _rdtsc() }
}

/// Estimate CPU frequency in Hz by comparing rdtsc against QueryPerformanceCounter over a short interval.
pub fn get_cpu_frequency2() -> u64 {
    let milliseconds_to_wait: u64 = 100;
    let os_freq = os_timer_frequency() as u64;
    let os_start = os_timer() as u64;
    let cpu_start = read_cpu_timer();

    let mut os_elapsed = 0;
    let os_wait_time = milliseconds_to_wait * os_freq / 1000;

    // Busy-wait for ~100ms
    while os_elapsed < os_wait_time {
        os_elapsed = os_timer() as u64 - os_start;
    }

    let cpu_end = read_cpu_timer();
    let cpu_elapsed = cpu_end - cpu_start;
    (cpu_elapsed * os_freq) / os_elapsed
}

/// Estimate the CPU's rdtsc frequency in Hz using `Instant` as the reference clock.
pub fn get_cpu_frequency() -> u64 {
    let start_time = Instant::now();
    let start_cycles = read_cpu_timer();

    // Busy-wait for ~100ms
    while start_time.elapsed().as_millis() < 100 {}

    let end_cycles = read_cpu_timer();
    let elapsed_nanos = start_time.elapsed().as_nanos() as u64;

    let cycles_per_nano = (end_cycles - start_cycles) as f64 / elapsed_nanos as f64;
    (cycles_per_nano * 1_000_000_000.0) as u64
}

/// Number of page faults incurred by the current process so far.
pub fn read_os_pagefault_count() -> u64 {
    let mut counters: ProcessMemoryCounters = unsafe { std::mem::zeroed() };
    counters.cb = std::mem::size_of::<ProcessMemoryCounters>() as u32;

    let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };

    if ok != 0 {
        counters.page_fault_count as u64
    } else {
        0
    }
}
