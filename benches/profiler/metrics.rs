#![allow(dead_code)]

use std::arch::x86_64::_rdtsc;
use std::time::Instant;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn QueryPerformanceFrequency(freq: *mut i64) -> i32;
    fn QueryPerformanceCounter(count: *mut i64) -> i32;
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
