use std::io::Write;

use crate::profiler::metrics::read_cpu_timer;

#[derive(PartialEq)]
enum TestMode {
    Testing,
    Completed,
    Error,
}

struct TestResults {
    count: u64,
    total_time: u64,
    max_time: u64,
    min_time: u64,
}

impl TestResults {
    fn new() -> Self {
        Self {
            count: 0,
            total_time: 0,
            max_time: 0,
            min_time: u64::MAX,
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
    bytes_accumulated_on_this_test: u64,
    ticks_accumulated_on_this_test: u64,

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
            bytes_accumulated_on_this_test: 0,
            ticks_accumulated_on_this_test: 0,
            results: TestResults::new(),
        }
    }

    pub fn is_testing(&mut self) -> bool {
        if self.mode != TestMode::Testing {
            return false;
        }

        let current_time = read_cpu_timer();

        if self.open_block_count > 0 {
            // A test iteration just finished: validate it, then record its timing.
            if self.open_block_count != self.close_block_count {
                self.mode = TestMode::Error;
                eprintln!("Unbalanced begin()/end() calls");
                return false;
            }
            if self.bytes_accumulated_on_this_test != self.target_processed_byte_count {
                self.mode = TestMode::Error;
                eprintln!(
                    "Processed byte count mismatch: got {}, expected {}",
                    self.bytes_accumulated_on_this_test, self.target_processed_byte_count
                );
                return false;
            }

            let elapsed = self.ticks_accumulated_on_this_test;
            self.results.count += 1;
            self.results.total_time += elapsed;
            if elapsed > self.results.max_time {
                self.results.max_time = elapsed;
            }
            if elapsed < self.results.min_time {
                self.results.min_time = elapsed;
                // Found a new fastest time, so extend the search window.
                self.test_started_at = current_time;

                // Overwrite the same line with the latest fastest time.
                print!("\r{}      ", self.format_time("Min", self.results.min_time));
                std::io::stdout().flush().ok();
            }

            // Reset the per-iteration accumulators for the next run.
            self.open_block_count = 0;
            self.close_block_count = 0;
            self.bytes_accumulated_on_this_test = 0;
            self.ticks_accumulated_on_this_test = 0;
        } else {
            // First call: start the search window.
            self.test_started_at = current_time;
        }

        if current_time - self.test_started_at > self.try_for_time {
            self.mode = TestMode::Completed;
            // The live line already shows the final Min; just end it and add the rest.
            println!();
            self.print_time("Max", self.results.max_time);
            if self.results.count > 0 {
                self.print_time("Avg", self.results.total_time / self.results.count);
            }
        }

        true
    }

    pub fn begin(&mut self) {
        self.open_block_count += 1;
        self.ticks_accumulated_on_this_test = self
            .ticks_accumulated_on_this_test
            .wrapping_sub(read_cpu_timer());
    }

    pub fn end(&mut self) {
        self.close_block_count += 1;
        self.ticks_accumulated_on_this_test = self
            .ticks_accumulated_on_this_test
            .wrapping_add(read_cpu_timer());
    }

    pub fn set_accumulated_bytes(&mut self, bytes: u64) {
        self.bytes_accumulated_on_this_test += bytes;
    }

    fn format_time(&self, label: &str, ticks: u64) -> String {
        let seconds = ticks as f64 / self.cpu_timer_freq as f64;
        let mut out = format!("{label}: {ticks} ({:.4}ms)", seconds * 1000.0);
        if self.target_processed_byte_count > 0 && seconds > 0.0 {
            let gigabyte = 1024.0 * 1024.0 * 1024.0;
            let bandwidth = self.target_processed_byte_count as f64 / (gigabyte * seconds);
            out.push_str(&format!(" {bandwidth:.4}gb/s"));
        }
        out
    }

    fn print_time(&self, label: &str, ticks: u64) {
        println!("{}", self.format_time(label, ticks));
    }
}
