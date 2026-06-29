use std::io::Write;

use crate::profiler::metrics::{read_cpu_timer, read_os_pagefault_count};

#[derive(PartialEq)]
enum TestMode {
    Testing,
    Completed,
    Error,
}

#[derive(Copy, Clone)]
enum ValueType {
    TestCount,

    CpuTimer,
    MemPageFaults,
    ByteCount,

    Count,
}

const VALUE_COUNT: usize = ValueType::Count as usize;

#[derive(Default, Copy, Clone)]
struct Value {
    e: [u64; VALUE_COUNT],
}

impl std::ops::Index<ValueType> for Value {
    type Output = u64;

    fn index(&self, value_type: ValueType) -> &u64 {
        &self.e[value_type as usize]
    }
}

impl std::ops::IndexMut<ValueType> for Value {
    fn index_mut(&mut self, value_type: ValueType) -> &mut u64 {
        &mut self.e[value_type as usize]
    }
}

struct TestResults {
    total: Value,
    max: Value,
    min: Value,
}

impl TestResults {
    fn new() -> Self {
        Self {
            total: Value::default(),
            max: Value::default(),
            min: Value {
                e: [u64::MAX; VALUE_COUNT],
            },
        }
    }
}

pub struct RepetitionTester {
    target_processed_byte_count: u64,
    cpu_timer_freq: u64,
    try_for_time: u64,
    test_started_at: u64,

    mode: TestMode,
    open_block_count: u64,
    close_block_count: u64,

    value_accumulated_on_test: Value,
    results: TestResults,
}

impl RepetitionTester {
    pub fn new(target_processed_byte_count: u64, cpu_timer_freq: u64, seconds_to_try: u64) -> Self {
        if cpu_timer_freq == 0 {
            panic!("Estimated CPU frequency is 0");
        }
        Self {
            target_processed_byte_count,
            cpu_timer_freq,
            try_for_time: seconds_to_try * cpu_timer_freq,
            test_started_at: 0,
            mode: TestMode::Testing,
            open_block_count: 0,
            close_block_count: 0,
            value_accumulated_on_test: Value::default(),
            results: TestResults::new(),
        }
    }

    pub fn reset(&mut self) {
        self.mode = TestMode::Testing;
        self.test_started_at = read_cpu_timer();
    }

    pub fn is_testing(&mut self) -> bool {
        if self.mode != TestMode::Testing {
            return false;
        }

        let current_time = read_cpu_timer();

        if self.open_block_count > 0 {
            if self.open_block_count != self.close_block_count {
                self.mode = TestMode::Error;
                eprintln!("Unbalanced begin()/end() calls");
                return false;
            }

            if self.value_accumulated_on_test[ValueType::ByteCount]
                != self.target_processed_byte_count
            {
                self.mode = TestMode::Error;
                eprintln!(
                    "Processed byte count mismatch: got {}, expected {}",
                    self.value_accumulated_on_test[ValueType::ByteCount],
                    self.target_processed_byte_count
                );
                return false;
            }

            self.value_accumulated_on_test[ValueType::TestCount] = 1;
            for (i, value) in self.value_accumulated_on_test.e.iter().enumerate() {
                self.results.total.e[i] += value;
            }

            if self.value_accumulated_on_test[ValueType::CpuTimer]
                > self.results.max[ValueType::CpuTimer]
            {
                self.results.max = self.value_accumulated_on_test;
                println!(
                    "-----------------> New MAX: {}, {}",
                    self.results.max[ValueType::CpuTimer],
                    self.results.max[ValueType::MemPageFaults]
                );
            }

            if self.value_accumulated_on_test[ValueType::CpuTimer]
                < self.results.min[ValueType::CpuTimer]
            {
                self.results.min = self.value_accumulated_on_test;
                // Found a new fastest time, so extend the search window.
                self.test_started_at = current_time;
                // Overwrite the same line with the latest fastest time.
                self.print_value("Min", self.results.min);
                println!();
                // print!("                                   \r");
                std::io::stdout().flush().ok();
            }

            // Reset the per-iteration accumulators for the next run.
            self.open_block_count = 0;
            self.close_block_count = 0;
            self.value_accumulated_on_test = Value::default();
        } else {
            // First call: start the search window.
            self.test_started_at = current_time;
        }

        if current_time - self.test_started_at > self.try_for_time {
            self.mode = TestMode::Completed;
            self.print_value("Min", self.results.min);
            println!();
            self.print_value("Max", self.results.max);
            println!();
            self.print_value("Avg", self.results.total);
            println!();
        }

        true
    }

    pub fn begin(&mut self) {
        self.open_block_count += 1;

        self.value_accumulated_on_test[ValueType::MemPageFaults] = self.value_accumulated_on_test
            [ValueType::MemPageFaults]
            .wrapping_sub(read_os_pagefault_count());

        self.value_accumulated_on_test[ValueType::CpuTimer] =
            self.value_accumulated_on_test[ValueType::CpuTimer].wrapping_sub(read_cpu_timer());
    }

    pub fn end(&mut self) {
        self.value_accumulated_on_test[ValueType::CpuTimer] =
            self.value_accumulated_on_test[ValueType::CpuTimer].wrapping_add(read_cpu_timer());

        self.value_accumulated_on_test[ValueType::MemPageFaults] = self.value_accumulated_on_test
            [ValueType::MemPageFaults]
            .wrapping_add(read_os_pagefault_count());

        self.close_block_count += 1;
    }

    pub fn set_accumulated_bytes(&mut self, bytes: u64) {
        self.value_accumulated_on_test[ValueType::ByteCount] += bytes;
    }

    fn print_value(&self, label: &str, value: Value) {
        let test_count = value[ValueType::TestCount];

        let mut elements: [f64; VALUE_COUNT] = [0.0; VALUE_COUNT];
        for (i, element) in value.e.iter().enumerate() {
            elements[i] = *element as f64 / test_count as f64;
        }

        let ticks = elements[ValueType::CpuTimer as usize];
        let seconds = ticks / self.cpu_timer_freq as f64;
        let mut out = format!("{label}: {ticks} ({:.4}ms)", 1000. * seconds);

        if self.target_processed_byte_count > 0 && seconds > 0. {
            let gigabyte = 1024.0 * 1024.0 * 1024.0;
            let bandwidth = self.target_processed_byte_count as f64 / (gigabyte * seconds);
            out.push_str(&format!(" {bandwidth:.4}gb/s"));
        }

        if elements[ValueType::MemPageFaults as usize] > 0. {
            let pagefaults = elements[ValueType::MemPageFaults as usize];
            let kib_per_pagefault = elements[ValueType::ByteCount as usize] / (pagefaults * 1024.);
            out.push_str(&format!(
                " PF: {pagefaults:.4} ({kib_per_pagefault:.4}k/fault)"
            ));
        }

        print!("{}", out);
    }
}
